import Device from "sap/ui/Device";
import UIComponent from "sap/ui/core/UIComponent";
import JSONModel from "sap/ui/model/json/JSONModel";

/**
 * @namespace de.regatta_hd.infoportal
 */
export default class Component extends UIComponent {
    private regattaModel: JSONModel;
    private filterModel: JSONModel;
    private contentDensityClass: string;

    public static metadata = {
        manifest: "json",
        interfaces: ["sap.ui.core.IAsyncContentCreation"],
    };

    public init(): void {
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
        const deviceModel: JSONModel = new JSONModel(Device).setDefaultBindingMode("OneWay");
        super.setModel(deviceModel, "device");

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
    }

    public getContentDensityClass(): string {
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