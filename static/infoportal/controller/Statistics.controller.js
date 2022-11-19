sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel"
], function (BaseController, JSONModel) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.ScoresTable", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("statistics").attachMatched(this._loadStatistics, this);
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    _loadStatistics: async function () {
      const oStatisticsModel = new JSONModel();
      await oStatisticsModel.loadData("/api/regattas/" + this.getRegattaId() + "/statistics");

      const oStatistics = oStatisticsModel.getData();
      oStatistics.items = [];
      oStatistics.items.push({ name: this.i18n("statistics.races.all"), value: oStatistics.races.all });
      oStatistics.items.push({ name: this.i18n("statistics.races.cancelled"), value: oStatistics.races.cancelled });
      oStatistics.items.push({ name: this.i18n("statistics.heats.all"), value: oStatistics.heats.all });
      oStatistics.items.push({ name: this.i18n("statistics.heats.official"), value: oStatistics.heats.official });
      oStatistics.items.push({ name: this.i18n("statistics.heats.finished"), value: oStatistics.heats.finished });
      oStatistics.items.push({ name: this.i18n("statistics.heats.started"), value: oStatistics.heats.started });
      oStatistics.items.push({ name: this.i18n("statistics.heats.pending"), value: oStatistics.heats.pending });
      oStatistics.items.push({ name: this.i18n("statistics.heats.cancelled"), value: oStatistics.heats.cancelled });

      this.setViewModel(oStatisticsModel, "statistics");
    }

  });
});