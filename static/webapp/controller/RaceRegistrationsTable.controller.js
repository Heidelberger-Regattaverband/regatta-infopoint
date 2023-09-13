sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "sap/m/MessageToast",
  "../model/Formatter"
], function (BaseController, JSONModel, MessageToast, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.RaceRegistrationsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.setViewModel(new JSONModel(), "raceRegistrations");
    },

    onBeforeRendering: async function () {
      await this._loadRaceModel()
    },

    onNavBack: function () {
      this.displayTarget("races");
    },

    onFirstPress: async function () {
      this.getEventBus().publish("race", "first", {});
      await this._loadRaceModel();
    },

    onPreviousPress: async function () {
      this.getEventBus().publish("race", "previous", {});
      await this._loadRaceModel();
    },

    onNextPress: async function () {
      this.getEventBus().publish("race", "next", {});
      await this._loadRaceModel();
    },

    onLastPress: async function () {
      this.getEventBus().publish("race", "last", {});
      await this._loadRaceModel();
    },

    onRefreshButtonPress: async function (oEvent) {
      const oSource = oEvent.getSource();
      oSource.setEnabled(false);
      await this._loadRaceModel();
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
      oSource.setEnabled(true);
    },

    _loadRaceModel: async function () {
      const oRace = this.getComponentModel("race");
      await this.updateJSONModel(this.getViewModel("raceRegistrations"), `/api/races/${oRace.getData().id}`, undefined);
    }
  });
});