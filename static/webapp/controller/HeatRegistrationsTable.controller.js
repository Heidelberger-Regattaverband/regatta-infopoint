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
      const mHeat = this.getComponentModel("heat").getData();
      await this.updateJSONModel(this.getViewModel("heatRegistrations"), `/api/heats/${mHeat.id}`, this.getView());
    }
  });
});