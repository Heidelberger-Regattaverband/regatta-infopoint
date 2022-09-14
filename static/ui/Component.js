sap.ui.define([
  "sap/ui/core/UIComponent",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/resource/ResourceModel"
], function (UIComponent, JSONModel, ResourceModel) {
  "use strict";
  return UIComponent.extend("de.regatta_hd.infopoint.Component", {
    metadata: {
      interfaces: ["sap.ui.core.IAsyncContentCreation"],
      manifest: "json"
    },
    init: function () {
      // call the init function of the parent
      UIComponent.prototype.init.apply(this, arguments);
      // set data model
      var oData = {
        recipient: {
          name: "World"
        }
      };
      var oModel = new JSONModel(oData);
      this.setModel(oModel);

      // var oInvoiceModel = new JSONModel();
      // oInvoiceModel.loadData("Invoices.json");
      // // oInvoiceModel.loadData("http://localhost:8080/api/heats");
      // this.setModel(oInvoiceModel, "invoice");
    }
  });
});
