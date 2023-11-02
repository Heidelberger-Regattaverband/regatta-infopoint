import Device from "sap/ui/Device";
import UIComponent from "sap/ui/core/UIComponent";
import JSONModel from "sap/ui/model/json/JSONModel";
/**
 * @namespace ui5.typescript.helloworld
 */
export default class Component extends UIComponent {
    private regattaModel: JSONModel;
    private filterModel: JSONModel;
    private contentDensityClass: String;

    public init(): void {
        super.init();
        // call the init function of the parent
        //   UIComponent.prototype.init.apply(this, arguments);

        // create the views based on the url/hash
        super.getRouter().initialize();

        this.regattaModel = new JSONModel();
        // ensure the active regatta is loaded, otherwise the regatta_id is unedfined
        this.regattaModel.loadData("/api/active_regatta", {}, false);
        super.setModel(this.regattaModel, "regatta");

        this.filterModel = new JSONModel();
        const iRegattaID = this.regattaModel.getData().id;
        this.filterModel.loadData(`/api/regattas/${iRegattaID}/filters`, {}, false);
        super.setModel(this.filterModel, "filters");

        // set device model
        const oDeviceModel = new JSONModel(Device);
        oDeviceModel.setDefaultBindingMode("OneWay");
        super.setModel(oDeviceModel, "device");

        const oUserModel = new JSONModel({
            authenticated: false, username: "anonymous", roles: []
        });
        oUserModel.setDefaultBindingMode("OneWay");
        super.setModel(oUserModel, "identity");

        // set initial heat model, required for navigation over heats
        super.setModel(new JSONModel(), "heat");

        // set initial race model, required for navigation over races
        super.setModel(new JSONModel(), "race");

        window.addEventListener('beforeunload', (oEvent) => {
            // Cancel the event as stated by the standard.
            oEvent.preventDefault();
            // Chrome requires returnValue to be set.
            oEvent.returnValue = '';
        });
    }

    public getContentDensityClass(): String {
        if (!this.contentDensityClass) {
            if (!Device.support.touch) {
                this.contentDensityClass = "sapUiSizeCompact";
            } else {
                this.contentDensityClass = "sapUiSizeCozy";
            }
        }
        return this.contentDensityClass;
    }

    public getRegattaId(): int {
        return this.regattaModel.getData().id;
    }
}