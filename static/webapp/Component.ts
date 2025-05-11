import ResourceBundle from "sap/base/i18n/ResourceBundle";
import Device from "sap/ui/Device";
import UIComponent from "sap/ui/core/UIComponent";
import JSONModel from "sap/ui/model/json/JSONModel";
import ResourceModel from "sap/ui/model/resource/ResourceModel";

/**
 * @namespace de.regatta_hd.infoportal
 */
export default class Component extends UIComponent {

    private contentDensityClass: string;
    private resourceBundle: ResourceBundle;

    private regattaModel?: JSONModel;
    private filtersModel?: JSONModel;
    private regattaModelPromise?: Promise<JSONModel>;
    private filtersModelPromise?: Promise<JSONModel>;

    static readonly metadata = {
        manifest: "json",
        interfaces: ["sap.ui.core.IAsyncContentCreation"]
    };

    async getActiveRegatta(): Promise<JSONModel> {
        if (this.regattaModelPromise) {
            return this.regattaModelPromise;
        }
        if (!this.regattaModel) {
            this.regattaModelPromise = this.loadActiveRegatta();
            this.regattaModel = await this.regattaModelPromise;
            delete this.regattaModelPromise;
        } else {
            console.debug("Active regatta already loaded");
        }
        return Promise.resolve(this.regattaModel);
    }

    async getFilters(): Promise<JSONModel> {
        if (this.filtersModelPromise) {
            return this.filtersModelPromise;
        }
        if (!this.filtersModel) {
            this.filtersModelPromise = this.loadFilters();
            this.filtersModel = await this.filtersModelPromise;
            delete this.filtersModelPromise;
        } else {
            console.debug("Filters already loaded");
        }
        return Promise.resolve(this.filtersModel);
    }

    init(): void {
        super.init();

        // create the views based on the url/hash
        super.getRouter().initialize();

        // set regatta and filters model
        this.getActiveRegatta().then((model: JSONModel) => {
            super.setModel(model, "regatta");

            this.getFilters().then((model: JSONModel) => {
                super.setModel(model, "filters");
            });
        })

        // set device model
        super.setModel(new JSONModel(Device).setDefaultBindingMode("OneWay"), "device");

        // set identity model
        const identityModel: JSONModel = new JSONModel({ authenticated: false, username: "anonymous", roles: [] }).setDefaultBindingMode("OneWay");
        super.setModel(identityModel, "identity");

        // set initial heat model, required for navigation over heats
        super.setModel(new JSONModel(), "heat");

        // set initial race model, required for navigation over races
        super.setModel(new JSONModel(), "race");

        window.addEventListener('beforeunload', (event: BeforeUnloadEvent) => {
            // Cancel the event as stated by the standard.
            event.preventDefault();
        });

        const bundle: ResourceBundle | Promise<ResourceBundle> = (super.getModel("i18n") as ResourceModel).getResourceBundle();
        if (bundle instanceof ResourceBundle) {
            this.resourceBundle = bundle;
        } else {
            bundle.then((bundle: ResourceBundle) => {
                this.resourceBundle = bundle;
            });
        }
    }

    /**
     * Returns the content density class according to the current device.
     * @returns {string} the content density class
     */
    getContentDensityClass(): string {
        if (!this.contentDensityClass) {
            if (!Device.support.touch) {
                this.contentDensityClass = "sapUiSizeCompact";
            } else {
                this.contentDensityClass = "sapUiSizeCozy";
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
}