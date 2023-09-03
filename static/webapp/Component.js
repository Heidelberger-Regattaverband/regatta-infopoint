sap.ui.define([
  "sap/ui/core/UIComponent",
  "sap/ui/model/json/JSONModel",
  "sap/ui/Device"
], function (UIComponent, JSONModel, Device) {
  "use strict";
  return UIComponent.extend("de.regatta_hd.infopoint.Component", {

    metadata: {
      interfaces: ["sap.ui.core.IAsyncContentCreation"],
      manifest: "json"
    },

    init: function () {
      // call the init function of the parent
      UIComponent.prototype.init.apply(this, arguments);

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
    },

    getContentDensityClass: function () {
      if (!this._sContentDensityClass) {
        if (!Device.support.touch) {
          this._sContentDensityClass = "sapUiSizeCompact";
        } else {
          this._sContentDensityClass = "sapUiSizeCozy";
        }
      }
      return this._sContentDensityClass;
    },

    getRegattaId: function () {
      return this._oRegattaModel.getData().id;
    }

  });
});