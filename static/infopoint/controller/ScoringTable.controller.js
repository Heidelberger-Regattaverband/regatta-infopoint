sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel"
], function (Controller, JSONModel) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.ScoresTable", {

    onInit: function () {
      const oComponent = this.getOwnerComponent();

      this.getView().addStyleClass(oComponent.getContentDensityClass());

      oComponent.getRouter().getRoute("scoring").attachPatternMatched(this._loadScoringModel, this);
    },

    onNavBack: function () {
      const oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("startpage", {}, true);
    },

    _loadScoringModel: function () {
      const oRegatta = this.getOwnerComponent().getModel("regatta").getData();
      const oScoringModel = new JSONModel();
      oScoringModel.loadData("/api/regattas/" + oRegatta.id + "/scoring");
      this.getOwnerComponent().setModel(oScoringModel, "scoring");
    }

  });
});