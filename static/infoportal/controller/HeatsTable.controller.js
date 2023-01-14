sap.ui.define([
  "de/regatta_hd/infopoint/controller/BaseTable.controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  "sap/ui/model/FilterOperator",
  "../model/Formatter"
], function (BaseTableController, JSONModel, Filter, FilterOperator, Formatter) {
  "use strict";

  return BaseTableController.extend("de.regatta_hd.infopoint.controller.HeatsTable", {

    formatter: Formatter,

    onInit: function () {
      this.init(this.getView().byId("heatsTable"), "heat");

      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("heats").attachMatched(this._loadHeatsModel, this);
    },

    onSelectionChange: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("heats");
        const oHeat = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());
        this.getOwnerComponent().setModel(new JSONModel(oHeat), "heat");

        this._loadRegistrationsModel(oHeat.id);
        this.displayTarget("heatRegistrations");
      }
    },

    onNavBack: function () {
      this._oHeatsModel = undefined;
      // reduce table growing threshold to improve performance next time table is shown
      this.oTable.setGrowingThreshold(30);
      this.navBack("startpage");
    },

    _loadHeatsModel: async function () {
      if (!this._oHeatsModel) {
        this._oHeatsModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/heats", this.oTable);
        this.setViewModel(this._oHeatsModel, "heats");
        this.applyFilters();
      }
    },

    _loadRegistrationsModel: async function (sHeatId) {
      const oModel = await this.getJSONModel("/api/heats/" + sHeatId + "/registrations", null);
      this.getOwnerComponent().setModel(oModel, "heatRegistrations");
    },

    onItemChanged: function (oItem) {
      this.getOwnerComponent().getModel("heat").setData(oItem);
      this._loadRegistrationsModel(oItem.id);
    },

    onHandleFilterButtonPressed: function (oEvent) {
      this.getViewSettingsDialog("de.regatta_hd.infopoint.view.HeatsFilterDialog")
        .then(function (oViewSettingsDialog) {
          oViewSettingsDialog.open();
        });
    },

    onFilterSearch: function (oEvent) {
      const aSearchFilters = [];
      const sQuery = oEvent.getParameter("query");
      if (sQuery) {
        aSearchFilters.push(
          new Filter({
            filters: [
              new Filter("race/number", FilterOperator.Contains, sQuery),
              new Filter("race/shortLabel", FilterOperator.Contains, sQuery),
              new Filter("race/longLabel", FilterOperator.Contains, sQuery)
            ],
            and: false
          }))
      }
      this.setSearchFilters(aSearchFilters);
      this.applyFilters();
    }
  });

});