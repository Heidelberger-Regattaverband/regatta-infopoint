import Device from "sap/ui/Device";
import UIComponent from "sap/ui/core/UIComponent";
import JSONModel from "sap/ui/model/json/JSONModel";
/**
 * @namespace ui5.typescript.helloworld
 */
export default class Component extends UIComponent {
    private _oRegattaModel: JSONModel;
    private _oFiltersModel: JSONModel;

    public init(): void {
        super.init();
        // call the init function of the parent
        //   UIComponent.prototype.init.apply(this, arguments);

        // create the views based on the url/hash
        this.getRouter().initialize();

        this._oRegattaModel = new JSONModel();
        // ensure the active regatta is loaded, otherwise the regatta_id is unedfined
        this._oRegattaModel.loadData("/api/active_regatta", {}, false);
        this.setModel(this._oRegattaModel, "regatta");

        this._oFiltersModel = new JSONModel();
        const iRegattaID = this._oRegattaModel.getData().id;
        this._oFiltersModel.loadData(`/api/regattas/${iRegattaID}/filters`, {}, false);
        this.setModel(this._oFiltersModel, "filters");

        // set device model
        const oDeviceModel = new JSONModel(Device);
        oDeviceModel.setDefaultBindingMode("OneWay");
        this.setModel(oDeviceModel, "device");

        const oUserModel = new JSONModel({
            authenticated: false, username: "anonymous", roles: []
        });
        oUserModel.setDefaultBindingMode("OneWay");
        this.setModel(oUserModel, "identity");

        // set initial heat model, required for navigation over heats
        this.setModel(new JSONModel(), "heat");

        // set initial race model, required for navigation over races
        this.setModel(new JSONModel(), "race");

        window.addEventListener('beforeunload', (oEvent) => {
            // Cancel the event as stated by the standard.
            oEvent.preventDefault();
            // Chrome requires returnValue to be set.
            oEvent.returnValue = '';
        });
    }
}