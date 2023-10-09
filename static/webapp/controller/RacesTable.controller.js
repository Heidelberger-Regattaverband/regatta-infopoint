sap.ui.define([
  "de/regatta_hd/infopoint/controller/BaseTable.controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  "sap/ui/model/FilterOperator",
  "sap/m/MessageToast",
  "sap/m/ViewSettingsItem",
  "sap/m/ViewSettingsFilterItem",
  "../model/Formatter"
], function (BaseTableController, JSONModel, Filter, FilterOperator, MessageToast, ViewSettingsItem, ViewSettingsFilterItem, Formatter) {
  "use strict";

  return BaseTableController.extend("de.regatta_hd.infopoint.controller.RacesTable", {

    formatter: Formatter,

    onInit: async function () {
      this.init(this.getView().byId("racesTable"), "race");

      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this._oRacesModel = await this.getJSONModel(`/api/regattas/${this.getRegattaId()}/races`, this.oTable);
      this.setViewModel(this._oRacesModel, "races");

      this.setComponentModel(new JSONModel(), "race");

      this.getRouter().getRoute("races").attachMatched(async (_) => await this._loadRacesModel(), this);

      const mFilters = this.getComponentModel("filters").getData();

      // initialize filter values
      const oViewSettingsDialog = await this.getViewSettingsDialog("de.regatta_hd.infopoint.view.RacesFilterDialog");

      if (mFilters.boatClasses) {
        const oBoatClassFilter = new ViewSettingsFilterItem({ multiSelect: true, key: "boatClass", text: "{i18n>common.boatClass}" });
        mFilters.boatClasses.forEach((boatClass) => {
          oBoatClassFilter.addItem(new ViewSettingsItem({ text: boatClass.caption + " (" + boatClass.abbreviation + ")", key: "boatClass/id___EQ___" + boatClass.id }));
        });
        oViewSettingsDialog.insertFilterItem(oBoatClassFilter, 0);
      }

      if (mFilters.ageClasses) {
        const oAgeClassFilter = new ViewSettingsFilterItem({ multiSelect: true, key: "ageClass", text: "{i18n>common.ageClass}" });
        mFilters.ageClasses.forEach((ageClass) => {
          oAgeClassFilter.addItem(new ViewSettingsItem({ text: ageClass.caption + " " + ageClass.suffix, key: "ageClass/id___EQ___" + ageClass.id }));
        });
        oViewSettingsDialog.insertFilterItem(oAgeClassFilter, 1);
      }

      if (mFilters.distances && mFilters.distances.length > 1) {
        const oDistancesFilter = new ViewSettingsFilterItem({ multiSelect: true, key: "distance", text: "{i18n>common.distance}" });
        mFilters.distances.forEach((distance) => {
          oDistancesFilter.addItem(new ViewSettingsItem({ text: distance + "m", key: "distance___EQ___" + distance }));
        });
        oViewSettingsDialog.insertFilterItem(oDistancesFilter, 3);
      }

      oViewSettingsDialog.getFilterItems().forEach(oFilterItem => {
        switch (oFilterItem.getKey()) {
          case 'distance':
            break;
          case 'boatClass':
            break;
          case 'ageClass':
            break;
        } // end switch
      });
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

        this.onItemChanged(oRace);
        this.displayTarget("raceRegistrations");
      }
    },

    onNavBack: function () {
      this.navBack("startpage");
      // reduce table growing threshold to improve performance next time table is shown
      this.oTable.setGrowingThreshold(30);
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
      const oSource = oEvent.getSource();
      oSource.setEnabled(false);
      await this._loadRacesModel();
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
      oSource.setEnabled(true);
    },

    _loadRacesModel: async function () {
      await this.updateJSONModel(this._oRacesModel, `/api/regattas/${this.getRegattaId()}/races`, this.oTable);
    },

    onItemChanged: function (oItem) {
      this.getComponentModel("race").setData(oItem);
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