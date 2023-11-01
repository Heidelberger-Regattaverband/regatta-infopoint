sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "sap/m/MessageToast",
  "../model/Formatter"
], function (BaseController, JSONModel, MessageToast, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.HeatRegistrationsTable", {

    formatter: Formatter,

    onInit: async function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.setViewModel(new JSONModel(), "heatRegistrations");

      this.getView().addEventDelegate({
        onBeforeShow: this.onBeforeShow,
      }, this);

      this.getEventBus().subscribe("heat", "itemChanged", this._onItemChanged, this);

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
      await this._loadHeatModel();
    },

    onNavBack: function () {
      const oData = this.getComponentModel("heat").getData();
      if (oData._nav.back) {
        this.displayTarget(oData._nav.back);
      } else {
        this.displayTarget("heats");
      }
    },

    onFirstPress: async function () {
      this.getEventBus().publish("heat", "first", {});
    },

    onPreviousPress: async function () {
      this.getEventBus().publish("heat", "previous", {});
    },

    onNextPress: async function () {
      this.getEventBus().publish("heat", "next", {});
    },

    onLastPress: async function () {
      this.getEventBus().publish("heat", "last", {});
    },

    onRefreshButtonPress: async function (oEvent) {
      const oSource = oEvent.getSource();
      oSource.setEnabled(false);
      await this._loadHeatModel();
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
      oSource.setEnabled(true);
    },

    _loadHeatModel: async function () {
      const mHeat = this.getComponentModel("heat").getData();
      await this.updateJSONModel(this.getViewModel("heatRegistrations"), `/api/heats/${mHeat.id}`, this.getView());
    },

    _onItemChanged: async function (channelId, eventId, parametersMap) {
      await this._loadHeatModel();
    }
  });
});