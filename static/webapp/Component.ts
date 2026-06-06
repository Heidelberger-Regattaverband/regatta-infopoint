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

    private notificationsTimer?: number;

    private contentDensityClass: string;
    private resourceBundle: ResourceBundle;

    private regattaModel?: JSONModel;
    private filtersModel?: JSONModel;
    private readonly notificationsModel: JSONModel = new JSONModel();
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
        this.regattaModel = await this.regattaModelPromise;
        return this.regattaModel;
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
        this.filtersModel = await this.filtersModelPromise;
        return this.filtersModel;
    }

    /**
     * Polling interval for notifications, in milliseconds.
     */
    private static readonly NOTIFICATIONS_POLL_INTERVAL_MS: number = 60_000;

    init(): void {
        super.init();

        // Register the SAP TNT icon font once, at component start-up.
        // (Was previously done by the Timekeeping controller on every instantiation.)
        Component.registerIconFonts();

        // 1. Set the static, synchronous models first so any view that renders
        //    immediately after router.initialize() finds them in place.
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

        // 2. Resolve the i18n resource bundle (sync or async, depending on UI5 config),
        //    cache it and inject it into the Formatter so static formatter methods
        //    can localise without performing a second (synchronous!) bundle load.
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

        // 3. Bootstrap async data, then initialize the router. We deliberately
        //    do NOT await this in `init()` (which is `void`), but we collect the
        //    initialization promise so consumers (and tests) could observe it.
        void this.bootstrap();
    }

    /**
     * Performs the asynchronous component bootstrap:
     * 1. loads regatta + filters in parallel,
     * 2. loads the initial notifications,
     * 3. starts the notifications polling timer,
     * 4. initializes the router.
     *
     * Errors at any step are logged but do not prevent the router from starting,
     * so the user always gets a navigable shell.
     */
    private async bootstrap(): Promise<void> {
        try {
            const [regattaModel, filtersModel]: [JSONModel, JSONModel] = await Promise.all([
                this.getActiveRegatta(),
                this.getFilters(),
            ]);
            super.setModel(regattaModel, "regatta");
            super.setModel(filtersModel, "filters");

            try {
                const notificationsModel: JSONModel = await this.loadNotifications();
                super.setModel(notificationsModel, "notifications");
            } catch (err: unknown) {
                console.error("Failed to load initial notifications", err as Error);
                // Still register the model so views can bind without errors.
                super.setModel(this.notificationsModel, "notifications");
            }

            this.startNotificationsPolling();
        } catch (err: unknown) {
            console.error("Component bootstrap failed", err as Error);
        } finally {
            // Always initialize the router so the app remains navigable
            // even if data calls failed.
            super.getRouter().initialize();
        }
    }

    /**
     * Starts the notifications polling timer. Idempotent: if a timer is already
     * running it will not be replaced.
     */
    private startNotificationsPolling(): void {
        if (this.notificationsTimer !== undefined) {
            return;
        }
        this.notificationsTimer = globalThis.setInterval(() => {
            this.loadNotifications().catch((err: unknown) => {
                console.error("Failed to refresh notifications", err as Error);
            });
        }, Component.NOTIFICATIONS_POLL_INTERVAL_MS);
    }

    exit(): void {
        if (this.notificationsTimer !== undefined) {
            globalThis.clearInterval(this.notificationsTimer);
            delete this.notificationsTimer;
        }
        super.exit();
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
     * @returns {Promise<sap.ui.model.json.JSONModel>} the filters model as a Promise
     */
    private async loadFilters(): Promise<JSONModel> {
        await this.getActiveRegatta();
        console.debug("Loading filters");
        const model: JSONModel = new JSONModel();
        const regattaId = this.regattaModel?.getData().id ?? -1;
        await model.loadData(`/api/regattas/${regattaId}/filters`);
        console.debug("Filters loaded");
        return model
    }

    private async loadNotifications(): Promise<JSONModel> {
        console.debug("Loading notifications");
        const regattaId = this.regattaModel?.getData().id ?? -1;
        await this.notificationsModel.loadData(`/api/regattas/${regattaId}/visible_notifications`);
        this.notificationsModel.refresh();
        console.debug("Notifications loaded");
        return this.notificationsModel;
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
