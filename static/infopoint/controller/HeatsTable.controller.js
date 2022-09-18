sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  'sap/ui/model/FilterOperator',
  "../model/Formatter",
  "../model/HeatLabelFormatter",
  "sap/f/library"
], function (Controller, JSONModel, Filter, FilterOperator, Formatter, HeatLabelFormatter, fioriLibrary) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.HeatsTable", {

    formatter: Formatter,

    heatLabelFormatter: HeatLabelFormatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      let oRegatta = this.getOwnerComponent().getModel("regatta").getData();

      let oIconTabBar = this.byId("heatsIconTabBar");
      let sKey = oIconTabBar.getSelectedKey();
      // this._setFilter(sKey);
    },

    onSelectionChange: function (oEvent) {
      let oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        let oBindingCtx = oSelectedItem.getBindingContext("heats");
        let oHeat = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());

        let oModel = new JSONModel();
        oModel.loadData("/api/heats/" + oHeat.id + "/registrations");

        this.getOwnerComponent().setModel(new JSONModel(oHeat), "heat");
        this.getOwnerComponent().setModel(oModel, "heatRegistration");

        let oFCL = this.getView().getParent().getParent();
        oFCL.setLayout(fioriLibrary.LayoutType.TwoColumnsMidExpanded);
      }
    },

    onFilterSelect: function (oEvent) {
      const sKey = oEvent.getParameter("key");
      this._setFilter(sKey);
    },

    _setFilter: function (sKey) {
      // Array to combine filters
      const aFilters = [new Filter({
        path: 'date',
        operator: FilterOperator.EQ,
        value1: sKey
      })];

      const oBinding = this.byId("heatsTable").getBinding("items");
      oBinding.filter(aFilters);
    }
  });
});