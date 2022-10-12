sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/core/routing/History"
], function (Controller, JSONModel, History) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.ScoresTable", {

    onInit: function () {
      const oComponent = this.getOwnerComponent();

      this.getView().addStyleClass(oComponent.getContentDensityClass());

      oComponent.getRouter().getRoute("scoring").attachPatternMatched(this._loadScoringModel, this);
    },

    onNavBack: function () {
      const sPreviousHash = History.getInstance().getPreviousHash();
      if (sPreviousHash) {
        window.history.go(-1);
      } else {
        this.getOwnerComponent().getRouter().navTo("startpage", {}, false /* history */);
      }
    },

    _loadScoringModel: function () {
      const oRegatta = this.getOwnerComponent().getModel("regatta").getData();
      const oScoringModel = new JSONModel();
      oScoringModel.loadData("/api/regattas/" + oRegatta.id + "/scoring");
      this.getOwnerComponent().setModel(oScoringModel, "scoring");
    }

  });
});