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

      this.getView().addEventDelegate({
        onBeforeShow: this.onBeforeShow,
      }, this);

      this.getEventBus().subscribe("race", "itemChanged", this._onItemChanged, this);

      window.addEventListener("keydown", async (event) => {
        switch (event.key) {
          case "F5":
            event.preventDefault();
            break;
          case "ArrowLeft":
            await this.onPreviousPress();
            break;
          case "ArrowRight":
            await this.onNextPress();
            break;
          case "ArrowUp":
            await this.onFirstPress();
            break;
          case "ArrowDown":
            await this.onLastPress();
            break;
        }
      });
    },

    onBeforeShow: async function () {
      await this._loadRaceModel();
    },

    onNavBack: function () {
      this.displayTarget("races");
    },

    onFirstPress: async function () {
      this.getEventBus().publish("race", "first", {});
    },

    onPreviousPress: async function () {
      this.getEventBus().publish("race", "previous", {});
    },

    onNextPress: async function () {
      this.getEventBus().publish("race", "next", {});
    },

    onLastPress: async function () {
      this.getEventBus().publish("race", "last", {});
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
    },

    _onItemChanged: async function (channelId, eventId, parametersMap) {
      await this._loadRaceModel();
    }
  });
});