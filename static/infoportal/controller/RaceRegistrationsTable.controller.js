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

      this.getRouter().getRoute("raceRegistrations").attachMatched(this._loadModel, this);
    },

    onNavBack: function () {
      this.navBack("races");
    },

    handlePrevious: function () {
      this.getEventBus().publish("race", "previous", {});
      this._loadModel();
    },

    handleNext: function () {
      this.getEventBus().publish("race", "next", {});
      this._loadModel();
    },

    _loadModel: function () {
      const oRace = this.getOwnerComponent().getModel("race").getData();
      const oModel = new JSONModel();
      oModel.loadData("/api/races/" + oRace.id + "/registrations");
      this.setViewModel(oModel, "raceRegistrations");
    }

  });
});