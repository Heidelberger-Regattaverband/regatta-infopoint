sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "../model/Formatter"
], function (BaseController, JSONModel, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.RacesTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("races").attachMatched(this._loadRacesModel, this);

      this.getEventBus().subscribe("race", "previous", this._onPreviousRaceEvent, this);
      this.getEventBus().subscribe("race", "next", this._onNextRaceEvent, this);

      this.racesTable = this.getView().byId("racesTable");
    },

    onItemPress: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("races");

        const oRace = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());
        this.getOwnerComponent().setModel(new JSONModel(oRace), "race");

        this.getRouter().navTo("raceRegistrations", {}, false /* history */);
      }
    },

    onNavBack: function () {
      this.oRacesModel = undefined;
      this.navBack("startpage");
    },

    _onPreviousRaceEvent: function (channelId, eventId, parametersMap) {
      const iIndex = this.racesTable.indexOfItem(this.racesTable.getSelectedItem());
      const iPreviousIndex = iIndex > 1 ? iIndex - 1 : 0;

      if (iIndex != iPreviousIndex) {
        this.racesTable.setSelectedItem(this.racesTable.getItems()[iPreviousIndex]);
        const oRace = this.getViewModel("races").getData()[iPreviousIndex];
        this.getOwnerComponent().getModel("race").setData(oRace);
      }
    },

    _onNextRaceEvent: function (channelId, eventId, parametersMap) {
      const aRaces = this.getViewModel("races").getData();

      this._growTable(aRaces);

      const iIndex = this.racesTable.indexOfItem(this.racesTable.getSelectedItem());
      const iNextIndex = iIndex < aRaces.length - 1 ? iIndex + 1 : iIndex;

      if (iIndex != iNextIndex) {
        this.racesTable.setSelectedItem(this.racesTable.getItems()[iNextIndex]);
        this.getOwnerComponent().getModel("race").setData(aRaces[iNextIndex]);
      }
    },

    _loadRacesModel: async function () {
      if (!this.oRacesModel) {
        const sNoDataText = this.racesTable.getNoDataText();
        this.racesTable.setNoDataText(this.i18n("common.loadingData"));
        this.oRacesModel = new JSONModel();
        await this.oRacesModel.loadData("/api/regattas/" + this.getRegattaId() + "/races");
        this.setViewModel(this.oRacesModel, "races");
        this.racesTable.setNoDataText(sNoDataText);
      }
    },

    _growTable: function (aRaces) {
      const iActual = this.racesTable.getGrowingInfo().actual;
      if (aRaces.length > iActual) {
        this.racesTable.setGrowingThreshold(iActual + 20);
        this.racesTable.getBinding("items").filter([]);
      }
    }
  });
});