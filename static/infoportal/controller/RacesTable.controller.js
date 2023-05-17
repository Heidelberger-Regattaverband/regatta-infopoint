sap.ui.define([
  "de/regatta_hd/infopoint/controller/BaseTable.controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  "sap/ui/model/FilterOperator",
  "sap/m/MessageToast",
  "../model/Formatter"
], function (BaseTableController, JSONModel, Filter, FilterOperator, MessageToast, Formatter) {
  "use strict";

  return BaseTableController.extend("de.regatta_hd.infopoint.controller.RacesTable", {

    formatter: Formatter,

    onInit: async function () {
      this.init(this.getView().byId("racesTable"), "race");

      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this._oRacesModel = await this.getJSONModel(`/api/regattas/${this.getRegattaId()}/races`, this.oTable);
      this.setViewModel(this._oRacesModel, "races");

      this._oRegistrationsModel = new JSONModel();
      this.getOwnerComponent().setModel(this._oRegistrationsModel, "raceRegistrations");

      this.getRouter().getRoute("races").attachMatched(async (_) => await this._loadRacesModel(), this);
    },

    onItemPress: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("races");
        const oRace = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());

        const iIndex = this.oTable.indexOfItem(oSelectedItem);
        const iCount = this.oTable.getItems().length;
        // store navigation meta information in selected item
        oRace._nav = { isFirst: iIndex == 0, isLast: iIndex == iCount - 1 };

        this.getOwnerComponent().setModel(new JSONModel(oRace), "race");

        this._loadRegistrationsModel(oRace.id);
        this.displayTarget("raceRegistrations");
      }
    },

    onNavBack: function () {
      this.navBack("startpage");
      // reduce table growing threshold to improve performance next time table is shown
      this.oTable.setGrowingThreshold(30);
      this._oRacesModel.setData({});
      this._oRegistrationsModel.setData({});
    },

    onFilterButtonPress: async function (oEvent) {
      const oViewSettingsDialog = await this.getViewSettingsDialog("de.regatta_hd.infopoint.view.RacesFilterDialog")
      oViewSettingsDialog.open();
    },

    onClearFilterPress: async function (oEvent) {
      const oViewSettingsDialog = await this.getViewSettingsDialog("de.regatta_hd.infopoint.view.RacesFilterDialog")
      oViewSettingsDialog.clearFilters();
      this.clearFilters();
      this.applyFilters();
    },

    onRefreshButtonPress: async function (oEvent) {
      await this._loadRacesModel();
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
    },

    _loadRacesModel: async function () {
      await this.updateJSONModel(this._oRacesModel, `/api/regattas/${this.getRegattaId()}/races`, this.oTable);
      this.applyFilters();
    },

    _loadRegistrationsModel: async function (sRaceId) {
      await this.updateJSONModel(this._oRegistrationsModel, `/api/races/${sRaceId}/registrations`, undefined);
    },

    onItemChanged: function (oItem) {
      this.getOwnerComponent().getModel("race").setData(oItem);
      this._loadRegistrationsModel(oItem.id);
    },

    onFilterSearch: function (oEvent) {
      const aSearchFilters = [];
      const sQuery = oEvent.getParameter("query").trim();
      if (sQuery) {
        aSearchFilters.push(
          new Filter({
            filters: [
              new Filter("number", FilterOperator.Contains, sQuery),
              new Filter("shortLabel", FilterOperator.Contains, sQuery),
              new Filter("longLabel", FilterOperator.Contains, sQuery),
              new Filter("comment", FilterOperator.Contains, sQuery)
            ],
            and: false
          }))
      }
      this.setSearchFilters(aSearchFilters);
      this.applyFilters();
    }

  });
});