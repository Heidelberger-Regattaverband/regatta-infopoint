sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel"
], function (BaseController, JSONModel) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Statistics", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("statistics").attachMatched(async (_) => await this._loadStatistics(), this);

      this._oStatisticsModel = new JSONModel();
      this._oStatisticsModel.setData({
        registrations: [],
        races: [],
        heats: []
      });
      this.setViewModel(this._oStatisticsModel, "statistics");

      this._oRegistrationsList = this.getView().byId("registrationsList");
      this._oRacesList = this.getView().byId("racesList");
      this._oHeatsList = this.getView().byId("heatsList");
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    onRefreshButtonPress: async function (oEvent) {
      await this._loadStatistics();
    },

    _loadStatistics: async function () {
      this._setBusy(true);

      // load statistic data from backend
      const oDataLoader = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/statistics");
      const oStatistics = oDataLoader.getData();

      // transform statistic data into human readable format
      const registrations = [];
      registrations.push({ name: this.i18n("common.overall"), value: oStatistics.registrations.all });
      registrations.push({ name: this.i18n("statistics.registrations.cancelled"), value: oStatistics.registrations.cancelled });
      registrations.push({ name: this.i18n("statistics.reportingClubs"), value: oStatistics.registrations.registeringClubs });
      registrations.push({ name: this.i18n("statistics.participatingClubs"), value: oStatistics.registrations.clubs });
      registrations.push({ name: this.i18n("common.athletes"), value: oStatistics.registrations.athletes });
      const races = [];
      races.push({ name: this.i18n("common.overall"), value: oStatistics.races.all });
      races.push({ name: this.i18n("common.cancelled"), value: oStatistics.races.cancelled });
      const heats = [];
      heats.push({ name: this.i18n("common.overall"), value: oStatistics.heats.all });
      heats.push({ name: this.i18n("heat.state.official"), value: oStatistics.heats.official });
      heats.push({ name: this.i18n("heat.state.finished"), value: oStatistics.heats.finished });
      heats.push({ name: this.i18n("heat.state.started"), value: oStatistics.heats.started });
      heats.push({ name: this.i18n("common.seeded"), value: oStatistics.heats.seeded });
      heats.push({ name: this.i18n("common.scheduled"), value: oStatistics.heats.scheduled });
      heats.push({ name: this.i18n("common.cancelled"), value: oStatistics.heats.cancelled });

      // update model
      this._oStatisticsModel.setProperty("/registrations", registrations);
      this._oStatisticsModel.setProperty("/races", races);
      this._oStatisticsModel.setProperty("/heats", heats);

      this._setBusy(false);
    },

    _setBusy: function (bBusy) {
      this._oRegistrationsList.setBusy(bBusy);
      this._oRacesList.setBusy(bBusy);
      this._oHeatsList.setBusy(bBusy);
    }

  });
});