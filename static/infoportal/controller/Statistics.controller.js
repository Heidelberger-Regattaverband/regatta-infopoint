sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller"
], function (BaseController) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Statistics", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("statistics").attachMatched(this._loadStatistics, this);

      this.registrationsList = this.getView().byId("registrationsList");
      this.racesList = this.getView().byId("racesList");
      this.heatsList = this.getView().byId("heatsList");
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    _loadStatistics: async function () {
      this._setBusy(true);

      const oStatisticsModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/statistics");

      const oStatistics = oStatisticsModel.getData();
      oStatistics.items = { registrations: [], races: [], heats: [] };
      oStatistics.items.registrations.push({ name: this.i18n("common.overall"), value: oStatistics.registrations.all });
      oStatistics.items.registrations.push({ name: this.i18n("statistics.registrations.cancelled"), value: oStatistics.registrations.cancelled });
      oStatistics.items.registrations.push({ name: this.i18n("statistics.reportingClubs"), value: oStatistics.registrations.registeringClubs });
      oStatistics.items.registrations.push({ name: this.i18n("statistics.participatingClubs"), value: oStatistics.registrations.clubs });
      oStatistics.items.registrations.push({ name: this.i18n("common.athletes"), value: oStatistics.registrations.athletes });
      oStatistics.items.races.push({ name: this.i18n("common.overall"), value: oStatistics.races.all });
      oStatistics.items.races.push({ name: this.i18n("statistics.races.cancelled"), value: oStatistics.races.cancelled });
      oStatistics.items.heats.push({ name: this.i18n("common.overall"), value: oStatistics.heats.all });
      oStatistics.items.heats.push({ name: this.i18n("heat.state.official"), value: oStatistics.heats.official });
      oStatistics.items.heats.push({ name: this.i18n("heat.state.finished"), value: oStatistics.heats.finished });
      oStatistics.items.heats.push({ name: this.i18n("heat.state.started"), value: oStatistics.heats.started });
      oStatistics.items.heats.push({ name: this.i18n("statistics.heats.pending"), value: oStatistics.heats.pending });
      oStatistics.items.heats.push({ name: this.i18n("statistics.heats.cancelled"), value: oStatistics.heats.cancelled });

      this.setViewModel(oStatisticsModel, "statistics");

      this._setBusy(false);
    },

    _setBusy: function (bBusy) {
      this.registrationsList.setBusy(bBusy);
      this.racesList.setBusy(bBusy);
      this.heatsList.setBusy(bBusy);
    }

  });
});