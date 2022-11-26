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

      this.getEventBus().subscribe("race", "first", this._onFirstRaceEvent, this);
      this.getEventBus().subscribe("race", "previous", this._onPreviousRaceEvent, this);
      this.getEventBus().subscribe("race", "next", this._onNextRaceEvent, this);
      this.getEventBus().subscribe("race", "last", this._onLastRaceEvent, this);

      this.racesTable = this.getView().byId("racesTable");
    },

    onItemPress: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("races");

        const oRace = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());
        this.getOwnerComponent().setModel(new JSONModel(oRace), "race");

        this._loadRegistrationsModel(oRace.id);
        this.displayTarget("raceRegistrations");
      }
    },

    onNavBack: function () {
      this.oRacesModel = undefined;
      this.navBack("startpage");
    },

    _setCurrentRace: function (iIndex) {
      this.racesTable.setSelectedItem(this.racesTable.getItems()[iIndex]);
      const oRace = this.getViewModel("races").getData()[iIndex];
      this.getOwnerComponent().getModel("race").setData(oRace);
      this._loadRegistrationsModel(oRace.id);
    },

    _onFirstRaceEvent: function (channelId, eventId, parametersMap) {
      this._setCurrentRace(0);
    },

    _onPreviousRaceEvent: function (channelId, eventId, parametersMap) {
      const iIndex = this.racesTable.indexOfItem(this.racesTable.getSelectedItem());
      const iPreviousIndex = iIndex > 1 ? iIndex - 1 : 0;

      if (iIndex != iPreviousIndex) {
        this._setCurrentRace(iPreviousIndex);
      }
    },

    _onNextRaceEvent: function (channelId, eventId, parametersMap) {
      const aRaces = this.getViewModel("races").getData();
      this._growTable(aRaces);

      const iIndex = this.racesTable.indexOfItem(this.racesTable.getSelectedItem());
      const iNextIndex = iIndex < aRaces.length - 1 ? iIndex + 1 : iIndex;

      if (iIndex != iNextIndex) {
        this._setCurrentRace(iNextIndex);
      }
    },

    _onLastRaceEvent: function (channelId, eventId, parametersMap) {
      const aRaces = this.getViewModel("races").getData();
      this._growTable(aRaces);
      this._setCurrentRace(aRaces.length - 1);
    },

    _loadRacesModel: async function () {
      if (!this.oRacesModel) {
        this.oRacesModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/races", this.racesTable);
        this.setViewModel(this.oRacesModel, "races");
      }
    },

    _loadRegistrationsModel: async function (sRaceId) {
      const oModel = await this.getJSONModel("/api/races/" + sRaceId + "/registrations");
      this.getOwnerComponent().setModel(oModel, "raceRegistrations");
    },

    _growTable: function (aRaces) {
      const iActual = this.racesTable.getGrowingInfo().actual;
      if (aRaces.length > iActual) {
        this.racesTable.setGrowingThreshold(iActual + 30);
        this.racesTable.getBinding("items").filter([]);
      }
    }

  });
});