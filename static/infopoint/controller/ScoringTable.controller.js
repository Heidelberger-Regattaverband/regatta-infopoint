sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "sap/f/library"
], function (Controller, JSONModel, fioriLibrary) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.ScoresTable", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      let oRegatta = this.getOwnerComponent().getModel("regatta").getData();

      let oModel = new JSONModel();
      oModel.loadData("/api/regattas/" + oRegatta.id + "/scores");
      this.getOwnerComponent().setModel(oModel, "scoring");
    },

    onNavBack: function () {
      const oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("startpage", {}, true);
    }

  });
});