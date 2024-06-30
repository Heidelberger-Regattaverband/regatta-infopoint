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
    private filterModel: JSONModel;
    private contentDensityClass: string;

    static metadata = {
        manifest: "json",
        interfaces: ["sap.ui.core.IAsyncContentCreation"]
    };
    resourceBundle: ResourceBundle;

    init(): void {
        super.init();

        // create the views based on the url/hash
        super.getRouter().initialize();

        this.regattaModel = new JSONModel();
        // ensure the active regatta is loaded, otherwise the regatta_id is unedfined
        this.regattaModel.loadData("/api/active_regatta", {}, false);
        super.setModel(this.regattaModel, "regatta");

        this.filterModel = new JSONModel();
        const regattaId = this.regattaModel.getData().id;
        this.filterModel.loadData(`/api/regattas/${regattaId}/filters`, {}, false);
        super.setModel(this.filterModel, "filters");

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
            // Chrome requires returnValue to be set.
            event.returnValue = '';
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