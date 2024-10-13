import ResourceBundle from "sap/base/i18n/ResourceBundle";
import Device from "sap/ui/Device";
import UIComponent from "sap/ui/core/UIComponent";
import JSONModel from "sap/ui/model/json/JSONModel";
import ResourceModel from "sap/ui/model/resource/ResourceModel";

/**
 * @namespace de.regatta_hd.infoportal
 */
export default class Component extends UIComponent {

    private regattaModel: JSONModel;
    private filtersModel: JSONModel;
    private regattaId: number = -1;
    private contentDensityClass: string;

    static metadata = {
        manifest: "json",
        interfaces: ["sap.ui.core.IAsyncContentCreation"]
    };
    resourceBundle: ResourceBundle;

    private async loadActiveRegatta(): Promise<JSONModel> {
        console.debug("Loading active regatta");
        const model: JSONModel = new JSONModel();
        await model.loadData("/api/active_regatta");
        console.debug("Active regatta loaded");
        return model;
    }

    private async loadFilters(): Promise<JSONModel> {
        await this.getActiveRegatta();
        console.debug("Loading filters");
        const model: JSONModel = new JSONModel();
        await model.loadData(`/api/regattas/${this.regattaId}/filters`);
        console.debug("Filters loaded");
        return model
    }

    async getActiveRegatta(): Promise<JSONModel> {
        if (!this.regattaModel) {
            this.regattaModel = await this.loadActiveRegatta();
            this.regattaId = this.regattaModel.getData().id;
            console.debug(`Active regatta: ${this.regattaId}`);
            return this.regattaModel;
        }
        console.debug("Active regatta already loaded");
        return Promise.resolve(this.regattaModel);
    }

    async getFilters(): Promise<JSONModel> {
        if (!this.filtersModel) {
            this.filtersModel = await this.loadFilters();
            return this.filtersModel;
        }
        console.debug("Filters already loaded");
        return Promise.resolve(this.filtersModel);
    }

    init(): void {
        super.init();

        // create the views based on the url/hash
        super.getRouter().initialize();

        // set regatta model
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
            this.resourceBundle = bundle as ResourceBundle;
        } else {
            (bundle as Promise<ResourceBundle>).then((bundle: ResourceBundle) => {
                this.resourceBundle = bundle;
            });
        }
    }

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

    getRegattaId(): number {
        return this.regattaModel.getData().id;
    }

    /**
     * Getter for the resource bundle.
     * @returns {sap.base.i18n.ResourceBundle} the resourceModel of the component
    */
    getResourceBundle(): ResourceBundle {
        return this.resourceBundle;
    }

}