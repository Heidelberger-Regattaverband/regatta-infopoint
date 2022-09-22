sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "sap/f/library"
], function (Controller, JSONModel, fioriLibrary) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.ScoresTable", {

    onInit: function () {
      const oComponent = this.getOwnerComponent();

      this.getView().addStyleClass(oComponent.getContentDensityClass());

      this.getOwnerComponent().getRouter().attachRouteMatched(function (oEvent) {
        if (oEvent.getParameter("name") === "scoring") {
          const oRegatta = oComponent.getModel("regatta").getData();
          this._loadScoringModel(oRegatta);
        }
      }, this);
    },

    onNavBack: function () {
      const oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("startpage", {}, true);
    },

    _loadScoringModel: function (oRegatta) {
      const oScoringModel = new JSONModel();
      oScoringModel.loadData("/api/regattas/" + oRegatta.id + "/scores");

      this.getOwnerComponent().setModel(oScoringModel, "scoring");
    }
  });
});