sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "../model/Formatter"
], function (BaseController, JSONModel, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.RaceRegistrationsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getOwnerComponent().getRouter().getRoute("raceRegistrations").attachMatched(this._onRouteMatched, this);
    },

    onNavBack: function () {
      this.navBack("races");
    },

    handlePrevious: function () {
      this.getEventBus().publish("race", "previous", {});
      this._iRaceId -= 1;
      this._loadModel();
    },

    handleNext: function () {
      this.getEventBus().publish("race", "next", {});
      this._iRaceId += 1;
      this._loadModel();
    },

    _onRouteMatched: function (oEvent) {
      const oArgs = oEvent.getParameter("arguments");
      this._iRaceId = parseInt(oArgs.raceId);
      this._loadModel();
    },

    _loadModel: function () {
      const oRaceModel = new JSONModel();
      oRaceModel.loadData("/api/races/" + this._iRaceId);
      this.getView().setModel(oRaceModel, "race");

      const oRegistrationsModel = new JSONModel();
      oRegistrationsModel.loadData("/api/races/" + this._iRaceId + "/registrations");
      this.getView().setModel(oRegistrationsModel, "raceRegistrations");
    }

  });
});