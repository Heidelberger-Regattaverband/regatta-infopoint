sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  'sap/ui/model/FilterOperator',
  "../model/Formatter",
  "sap/f/library"
], function (Controller, JSONModel, Filter, FilterOperator, Formatter, fioriLibrary) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.HeatsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getOwnerComponent().getRouter().getRoute("heats").attachMatched(function () {
        const oIconTabBar = this.byId("heatsIconTabBar");
        oIconTabBar.setSelectedKey("all");
      }, this);
    },

    onSelectionChange: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("heats");
        const oHeat = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());

        const oModel = new JSONModel();
        oModel.loadData("/api/heats/" + oHeat.id + "/registrations");

        this.getOwnerComponent().setModel(new JSONModel(oHeat), "heat");
        this.getOwnerComponent().setModel(oModel, "heatRegistration");

        const oFCL = this.getView().getParent().getParent();
        oFCL.setLayout(fioriLibrary.LayoutType.TwoColumnsMidExpanded);
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
    }
  });
});