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
      Promise.resolve(this._oRegattaModel.loadData("/api/active_regatta"));
      this.setModel(this._oRegattaModel, "regatta");

      // set device model
      const oDeviceModel = new JSONModel(Device);
      oDeviceModel.setDefaultBindingMode("OneWay");
      this.setModel(oDeviceModel, "device");
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
