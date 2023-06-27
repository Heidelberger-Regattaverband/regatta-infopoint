sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "sap/m/MessageToast",
  "../model/Formatter"
], function (BaseController, JSONModel, MessageToast, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Statistics", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("statistics").attachMatched(async (_) => await this._loadStatistics(), this);

      this._oStatisticsModel = new JSONModel();
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
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
    },

    _loadStatistics: async function () {
      this._setBusy(true);

      // load statistic data from backend
      const oDataLoader = await this.getJSONModel(`/api/regattas/${this.getRegattaId()}/statistics`, undefined);
      const oStatistics = oDataLoader.getData();

      // transform statistic data into human readable format
      const registrations = [];
      const seats = oStatistics.registrations.seats + oStatistics.registrations.seatsCox;
      registrations.push({ name: this.i18n("common.overall", undefined), value: oStatistics.registrations.all });
      registrations.push({ name: this.i18n("statistics.registrations.cancelled", undefined), value: oStatistics.registrations.cancelled });
      registrations.push({ name: this.i18n("statistics.reportingClubs", undefined), value: oStatistics.registrations.registeringClubs });
      registrations.push({ name: this.i18n("statistics.participatingClubs", undefined), value: oStatistics.registrations.clubs });
      registrations.push({ name: this.i18n("common.athletes", undefined), value: oStatistics.registrations.athletes });
      registrations.push({ name: this.i18n("common.seats", undefined), value: seats });
      const races = [];
      races.push({ name: this.i18n("common.overall", undefined), value: oStatistics.races.all });
      races.push({ name: this.i18n("common.cancelled", undefined), value: oStatistics.races.cancelled });
      const heats = [];
      heats.push({ name: this.i18n("common.overall", undefined), value: oStatistics.heats.all });
      heats.push({ name: this.i18n("heat.state.official", undefined), value: oStatistics.heats.official });
      heats.push({ name: this.i18n("heat.state.finished", undefined), value: oStatistics.heats.finished });
      heats.push({ name: this.i18n("heat.state.started", undefined), value: oStatistics.heats.started });
      heats.push({ name: this.i18n("common.seeded", undefined), value: oStatistics.heats.seeded });
      heats.push({ name: this.i18n("common.scheduled", undefined), value: oStatistics.heats.scheduled });
      heats.push({ name: this.i18n("common.cancelled", undefined), value: oStatistics.heats.cancelled });

      const oldestWoman = oStatistics.athletes.oldestWoman;
      const oldestMan = oStatistics.athletes.oldestMan;
      const athletes = [];
      athletes.push({ name: this.i18n("statistics.athletes.oldestWoman", undefined), value: Formatter.athleteLabel(oldestWoman) });
      athletes.push({ name: this.i18n("statistics.athletes.oldestMan", undefined), value: Formatter.athleteLabel(oldestMan) });

      // update model
      this._oStatisticsModel.setProperty("/registrations", registrations);
      this._oStatisticsModel.setProperty("/races", races);
      this._oStatisticsModel.setProperty("/heats", heats);
      this._oStatisticsModel.setProperty("/athletes", athletes);

      this._setBusy(false);
    },

    _setBusy: function (bBusy) {
      this._oRegistrationsList.setBusy(bBusy);
      this._oRacesList.setBusy(bBusy);
      this._oHeatsList.setBusy(bBusy);
    }

  });
});