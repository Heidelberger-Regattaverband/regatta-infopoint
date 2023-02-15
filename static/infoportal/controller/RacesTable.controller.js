sap.ui.define([
  "de/regatta_hd/infopoint/controller/BaseTable.controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  "sap/ui/model/FilterOperator",
  "../model/Formatter"
], function (BaseTableController, JSONModel, Filter, FilterOperator, Formatter) {
  "use strict";

  return BaseTableController.extend("de.regatta_hd.infopoint.controller.RacesTable", {

    formatter: Formatter,

    onInit: function () {
      this.init(this.getView().byId("racesTable"), "race");

      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("races").attachMatched(this._loadRacesModel, this);
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
      this._oRacesModel = undefined;
      // reduce table growing threshold to improve performance next time table is shown
      this.oTable.setGrowingThreshold(30);
      this.navBack("startpage");
    },

    onFilterButtonPress: function (oEvent) {
      this.getViewSettingsDialog("de.regatta_hd.infopoint.view.RacesFilterDialog")
        .then(function (oViewSettingsDialog) {
          oViewSettingsDialog.open();
        });
    },

    onRefreshButtonPress: async function (oEvent) {
      await this.updateJSONModel(this._oRacesModel, "/api/regattas/" + this.getRegattaId() + "/races", this.oTable);
    },

    _loadRacesModel: async function () {
      if (!this._oRacesModel) {
        this._oRacesModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/races", this.oTable);
        this.setViewModel(this._oRacesModel, "races");
        this.applyFilters();
      }
    },

    _loadRegistrationsModel: async function (sRaceId) {
      if (!this._oRegistrationsModel) {
        this._oRegistrationsModel = await this.getJSONModel("/api/races/" + sRaceId + "/registrations", undefined);
        this.getOwnerComponent().setModel(this._oRegistrationsModel, "raceRegistrations");
      } else {
        await this.updateJSONModel(this._oRegistrationsModel, "/api/races/" + sRaceId + "/registrations", undefined);
      }
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
              new Filter("longLabel", FilterOperator.Contains, sQuery)
            ],
            and: false
          }))
      }
      this.setSearchFilters(aSearchFilters);
      this.applyFilters();
    }

  });
});