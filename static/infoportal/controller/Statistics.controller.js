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
      await oStatisticsModel.loadData("/api/regattas/" + this.getOwnerComponent().getRegattaId() + "/statistics");

      const oStatistics = oStatisticsModel.getData();
      oStatistics.items = [];
      oStatistics.items.push({ name: "Rennen gesamnt", value: oStatistics.races.all });
      oStatistics.items.push({ name: "Rennen abgesagt", value: oStatistics.races.cancelled });
      oStatistics.items.push({ name: "Laeufe gesamnt", value: oStatistics.heats.all });
      oStatistics.items.push({ name: "Laeufe abgesagt", value: oStatistics.heats.cancelled });
      oStatistics.items.push({ name: "Laeufe gestarted", value: oStatistics.heats.started });
      oStatistics.items.push({ name: "Laeufe beendet", value: oStatistics.heats.finished });
      oStatistics.items.push({ name: "Laeufe offizielle", value: oStatistics.heats.official });
      oStatistics.items.push({ name: "Laeufe bevorstehend", value: oStatistics.heats.pending });

      this.setViewModel(oStatisticsModel, "statistics");
    }

  });
});