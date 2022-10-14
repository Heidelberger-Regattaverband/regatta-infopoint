sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/core/routing/History",
  "../model/Formatter"
], function (Controller, JSONModel, History, Formatter) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.RaceRegistrationsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getOwnerComponent().getRouter().getRoute("raceRegistrations").attachMatched(this._loadModel, this);
    },

    onNavBack: function () {
      const sPreviousHash = History.getInstance().getPreviousHash();
      if (sPreviousHash) {
        window.history.go(-1);
      } else {
        this.getOwnerComponent().getRouter().navTo("races", {}, false /* history */);
      }
    },

    _loadModel: function () {
      const oRace = this.getOwnerComponent().getModel("race").getData();
      const oModel = new JSONModel();
      oModel.loadData("/api/races/" + oRace.id + "/registrations");
      this.getOwnerComponent().setModel(oModel, "raceRegistrations");
    }

  });
});