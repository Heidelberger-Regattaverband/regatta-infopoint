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

    _loadStatistics: function () {
      const oStatisticsModel = new JSONModel();
      oStatisticsModel.loadData("/api/regattas/" + this.getOwnerComponent().getRegattaId() + "/statistics");
      this.getOwnerComponent().setModel(oStatisticsModel, "statistics");
    }

  });
});