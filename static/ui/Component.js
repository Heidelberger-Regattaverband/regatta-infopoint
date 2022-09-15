sap.ui.define([
  "sap/ui/core/UIComponent",
  "sap/ui/Device"
], function (UIComponent, Device) {
  "use strict";
  return UIComponent.extend("de.regatta_hd.infopoint.Component", {

    metadata: {
      interfaces: ["sap.ui.core.IAsyncContentCreation"],
      manifest: "json"
    },

    init: function () {
      // call the init function of the parent
      UIComponent.prototype.init.apply(this, arguments);

      // var oInvoiceModel = new JSONModel();
      // oInvoiceModel.loadData("Invoices.json");
      // // oInvoiceModel.loadData("http://localhost:8080/api/heats");
      // this.setModel(oInvoiceModel, "invoice");
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
    }

  });
});
