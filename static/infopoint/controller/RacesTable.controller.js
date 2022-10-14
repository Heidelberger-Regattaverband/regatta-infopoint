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

      this.getEventBus().subscribe("race", "previous", this.previousRace, this);
      this.getEventBus().subscribe("race", "next", this.nextRace, this);

      this.racesTable = this.getView().byId("racesTable");
    },

    onItemPress: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("races");

        const oRace = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());
        this.getOwnerComponent().setModel(new JSONModel(oRace), "race");

        this.getOwnerComponent().getRouter().navTo("raceRegistrations", {}, false /* history */);
      }
    },

    onNavBack: function () {
      this.oRacesModel = undefined;
      this.navBack("startpage");
    },

    _loadRacesModel: function () {
      if (!this.oRacesModel) {
        const oRegatta = this.getOwnerComponent().getModel("regatta").getData();
        this.oRacesModel = new JSONModel();
        this.oRacesModel.loadData("/api/regattas/" + oRegatta.id + "/races");
        this.setViewModel(this.oRacesModel, "races");
      }
    },

    previousRace: function (channelId, eventId, parametersMap) {
      const iIndex = this.racesTable.indexOfItem(this.racesTable.getSelectedItem());
      const iPreviousIndex = iIndex > 1 ? iIndex - 1 : 0;

      if (iIndex != iPreviousIndex) {
        this.racesTable.setSelectedItem(this.racesTable.getItems()[iPreviousIndex]);
        const oRace = this.getViewModel("races").getData()[iPreviousIndex];
        this.getOwnerComponent().getModel("race").setData(oRace);
      }
    },

    nextRace: function (channelId, eventId, parametersMap) {
      const aRaces = this.getViewModel("races").getData();
      const iIndex = this.racesTable.indexOfItem(this.racesTable.getSelectedItem());
      const iNextIndex = iIndex < aRaces.length - 1 ? iIndex + 1 : iIndex;

      if (iIndex != iNextIndex) {
        this.racesTable.setSelectedItem(this.racesTable.getItems()[iNextIndex]);
        const oRace = aRaces[iNextIndex];
        this.getOwnerComponent().getModel("race").setData(oRace);
      }
    }

  });
});