import ResourceBundle from "sap/base/i18n/ResourceBundle";
import Device from "sap/ui/Device";
import IconPool from "sap/ui/core/IconPool";
import UIComponent from "sap/ui/core/UIComponent";
import JSONModel from "sap/ui/model/json/JSONModel";
import ResourceModel from "sap/ui/model/resource/ResourceModel";
import HeatsTableController from "./controller/HeatsTable.controller";
import RacesTableController from "./controller/RacesTable.controller";
import Formatter from "./model/Formatter";
import { NavigationData } from "./model/types";

/**
 * @namespace de.regatta_hd.infoportal
 */
export default class Component extends UIComponent {

    private contentDensityClass: string;
    private resourceBundle: ResourceBundle;

    // Memoised promises ensure concurrent callers share a single in-flight request and the cached model thereafter
    private regattaModelPromise?: Promise<JSONModel>;
    private filtersModelPromise?: Promise<JSONModel>;

    static readonly metadata = {
        manifest: "json",
        interfaces: ["sap.ui.core.IAsyncContentCreation"]
    };

    /**
     * Returns the active-regatta {@link JSONModel}.
     *
     * Concurrent callers share the same in-flight request via a memoised
     * promise; later callers receive the resolved value immediately.
     *
     * On failure (e.g. transient backend outage during bootstrap) the cached
     * promise is **invalidated** so the next call retries the network request,
     * rather than handing every subsequent caller the same rejected promise.
     */
    async getActiveRegatta(): Promise<JSONModel> {
        this.regattaModelPromise ??= this.loadActiveRegatta().catch((err: unknown) => {
            // Reset the cache so the next caller can retry with a fresh request.
            this.regattaModelPromise = undefined;
            throw err;
        });
        return await this.regattaModelPromise;
    }

    /**
     * Returns the filters {@link JSONModel} for the active regatta.
     *
     * Same memoisation + failure-invalidation contract as {@link getActiveRegatta}.
     */
    async getFilters(): Promise<JSONModel> {
        this.filtersModelPromise ??= this.loadFilters().catch((err: unknown) => {
            // Reset the cache so the next caller can retry with a fresh request.
            this.filtersModelPromise = undefined;
            throw err;
        });
        return await this.filtersModelPromise;
    }

    init(): void {
        super.init();

        // 1. Register all synchronous, view-bindable models *before* the router
        //    starts. The router's `initialize()` triggers route matching, which
        //    instantiates the matched view and its controller; that view may
        //    bind against any of these models on its first render.
        super.setModel(new JSONModel(Device).setDefaultBindingMode("OneWay"), "device");

        const identityModel: JSONModel = new JSONModel({ authenticated: false, username: "anonymous", roles: [] }).setDefaultBindingMode("OneWay");
        super.setModel(identityModel, "identity");

        // initial heat / race models, required for navigation over heats and races
        super.setModel(new JSONModel(), HeatsTableController.HEAT_MODEL);
        super.setModel(new JSONModel(), RacesTableController.RACE_MODEL);

        // Dedicated navigation-state models for the race/heat detail views.
        // The state ({@link NavigationData}) is intentionally kept *separate*
        // from the bound data models so that backend payloads are never
        // mutated with UI metadata.
        const initialNavigationData: NavigationData = { isFirst: false, isLast: false, disabled: false, back: undefined };
        super.setModel(new JSONModel({ ...initialNavigationData }), RacesTableController.RACE_NAV_MODEL);
        super.setModel(new JSONModel({ ...initialNavigationData }), HeatsTableController.HEAT_NAV_MODEL);

        // 2. Initialize the router as early as possible — *immediately* after
        //    all view-bindable models are registered. This is the earliest
        //    correct point: the static models above are needed by the first
        //    matched view, while everything below (icon-font registration,
        //    i18n bundle resolution, the `beforeunload` listener, and the async
        //    backend fetches in `bootstrap()`) is independent of routing and
        //    can run concurrently with — or after — the first paint.
        super.getRouter().initialize();

        // 3. Side-effects that do not need to precede the first route match.

        // Register the SAP TNT icon font once, at component start-up.
        Component.registerIconFonts();

        // Resolve the i18n resource bundle (sync or async, depending on UI5
        // config), cache it and inject it into the Formatter so static
        // formatter methods can localise without performing a second
        // (synchronous!) bundle load. Until the bundle is available, the
        // Formatter falls back to returning the i18n key — matching the
        // behaviour of UI5's `{i18n>...}` bindings before bundle resolution.

        const bundle: ResourceBundle | Promise<ResourceBundle> = (super.getModel("i18n") as ResourceModel).getResourceBundle();
        if (bundle instanceof ResourceBundle) {
            this.resourceBundle = bundle;
            Formatter.init(bundle);
        } else {
            bundle.then((resolved: ResourceBundle) => {
                this.resourceBundle = resolved;
                Formatter.init(resolved);
            }, (err: unknown) => {
                console.error("Failed to load i18n resource bundle", err as Error);
            });
        }

        globalThis.addEventListener('beforeunload', (event: BeforeUnloadEvent) => {
            // Cancel the event as stated by the standard.
            event.preventDefault();
        });

        // 4. Bootstrap async data in the background. Errors are logged but
        //    cannot block the navigable shell from rendering.
        void this.bootstrap();
    }

    /**
     * Performs the asynchronous component bootstrap (runs in the background
     * after the router is already initialized):
     * loads regatta + filters in parallel and registers them as
     * component-scoped models.
     *
     * Errors at any step are logged. The router is **not** started here — it
     * was already initialized synchronously by {@link init}, so the user gets
     * a navigable shell regardless of backend availability.
     */
    private async bootstrap(): Promise<void> {
        try {
            const [regattaModel, filtersModel]: [JSONModel, JSONModel] = await Promise.all([
                this.getActiveRegatta(),
                this.getFilters(),
            ]);
            super.setModel(regattaModel, "regatta");
            super.setModel(filtersModel, "filters");
        } catch (err: unknown) {
            console.error("Failed to load regatta/filters during bootstrap", err as Error);
        }
    }

    /**
     * Returns the content density class according to the current device.
     * @returns {string} the content density class
     */
    getContentDensityClass(): string {
        if (!this.contentDensityClass) {
            if (Device.support.touch) {
                this.contentDensityClass = "sapUiSizeCozy";
            } else {
                this.contentDensityClass = "sapUiSizeCompact";
            }
        }
        return this.contentDensityClass;
    }

    /**
     * Getter for the resource bundle.
     * @returns {sap.base.i18n.ResourceBundle} the resourceModel of the component
     */
    getResourceBundle(): ResourceBundle {
        return this.resourceBundle;
    }

    /**
     * Loads the active regatta into a JSONModel from the server and returns it as a Promise.
     * @returns {Promise<sap.ui.model.json.JSONModel>} the active regatta model as a Promise
     */
    private async loadActiveRegatta(): Promise<JSONModel> {
        console.debug("Loading active regatta");
        const model: JSONModel = new JSONModel();
        await model.loadData("/api/active_regatta");
        console.debug("Active regatta loaded");
        return model;
    }

    /**
     * Loads the filters into a JSONModel for the active regatta from the server and returns it as a Promise.
     *
     * Reads the regatta id from the resolved JSONModel directly rather than
     * from the side-channel `this.regattaModel` field, so the call cannot
     * silently fall through to `/api/regattas/-1/filters` if the field
     * assignment in {@link getActiveRegatta} is ever decoupled from the
     * promise resolution.
     *
     * @returns {Promise<sap.ui.model.json.JSONModel>} the filters model as a Promise
     */
    private async loadFilters(): Promise<JSONModel> {
        const regattaModel: JSONModel = await this.getActiveRegatta();
        console.debug("Loading filters");
        const filtersModel: JSONModel = new JSONModel();
        const regattaId = regattaModel.getData().id;
        await filtersModel.loadData(`/api/regattas/${regattaId}/filters`);
        console.debug("Filters loaded");
        return filtersModel;
    }

    /**
     * Registers icon fonts used by the application. `IconPool.registerFont` is
     * idempotent, but calling it once at component start-up — instead of in
     * every controller's `onInit` — is cheaper and keeps the side-effect in a
     * single, discoverable place.
     */
    private static registerIconFonts(): void {
        IconPool.registerFont({
            fontFamily: "SAP-icons-TNT",
            fontURI: sap.ui.require.toUrl("sap/tnt/themes/base/fonts/"),
        });
    }
}
