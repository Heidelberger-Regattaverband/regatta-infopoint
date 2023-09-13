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

      // this.getRouter().getRoute("heatRegistrations").attachMatched(async (_) => await this._loadHeatModel(), this);
    },

    onBeforeRendering: async function () {
      await this._loadHeatModel();
    },

    onNavBack: function () {
      // await this.navBack("heats");
      this.displayTarget("heats");
    },

    onFirstPress: async function () {
      this.getEventBus().publish("heat", "first", {});
      await this._loadHeatModel();
    },

    onPreviousPress: async function () {
      this.getEventBus().publish("heat", "previous", {});
      await this._loadHeatModel();
    },

    onNextPress: async function () {
      this.getEventBus().publish("heat", "next", {});
      await this._loadHeatModel();
    },

    onLastPress: async function () {
      this.getEventBus().publish("heat", "last", {});
      await this._loadHeatModel();
    },

    onRefreshButtonPress: async function (oEvent) {
      const oSource = oEvent.getSource();
      oSource.setEnabled(false);
      await this._loadHeatModel();
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
      oSource.setEnabled(true);
    },

    _loadHeatModel: async function () {
      const oHeat = this.getComponentModel("heat");
      if (oHeat) {
        await this.updateJSONModel(this.getViewModel("heatRegistrations"), `/api/heats/${oHeat.getData().id}`, this.getView());
      } else {
        this.onNavBack();
      }
    },

  });
});