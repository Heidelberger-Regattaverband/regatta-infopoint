sap.ui.define([
  "de/regatta_hd/infopoint/controller/BaseTable.controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  'sap/ui/model/FilterOperator',
  "../model/Formatter"
], function (BaseTableController, JSONModel, Filter, FilterOperator, Formatter) {
  "use strict";

  return BaseTableController.extend("de.regatta_hd.infopoint.controller.HeatsTable", {

    formatter: Formatter,

    onInit: function () {
      this.init(this.getView().byId("heatsTable"), "heat");

      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("heats").attachMatched(function () {
        this.byId("heatsIconTabBar").setSelectedKey("all");
        this._loadHeatsModel();
      }, this);
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

    onFilterSelect: function (oEvent) {
      const sKey = oEvent.getParameter("key");
      this._setFilter(sKey);
    },

    _setFilter: function (sKey) {
      let aFilters = [];
      if (sKey != "all") {
        aFilters.push(new Filter({
          path: 'date',
          operator: FilterOperator.EQ,
          value1: sKey
        }));
      }

      const oBinding = this.byId("heatsTable").getBinding("items");
      oBinding.filter(aFilters);
      this.setFilters(aFilters);
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
      }
    },

    _loadRegistrationsModel: async function (sHeatId) {
      const oModel = await this.getJSONModel("/api/heats/" + sHeatId + "/registrations", null);
      this.getOwnerComponent().setModel(oModel, "heatRegistrations");
    },

    onItemChanged: function (oItem) {
      this.getOwnerComponent().getModel("heat").setData(oItem);
      this._loadRegistrationsModel(oItem.id);
    }
  });

});