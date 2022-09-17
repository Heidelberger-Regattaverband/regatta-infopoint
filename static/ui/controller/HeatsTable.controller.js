sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  'sap/ui/model/FilterOperator',
  "../model/StateLabelFormatter",
  "../model/Formatter",
  "../model/HeatLabelFormatter",
  "sap/f/library"
], function (Controller, JSONModel, Filter, FilterOperator, StateLabelFormatter, Formatter, HeatLabelFormatter, fioriLibrary) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.HeatsTable", {

    stateLabelFormatter: StateLabelFormatter,

    formatter: Formatter,

    heatLabelFormatter: HeatLabelFormatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());
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
      let oBinding = this.byId("heatsTable").getBinding("items");
      let sKey = oEvent.getParameter("key");
      // Array to combine filters
      let aFilters = [];
      // debugger;
      aFilters.push(
        new Filter({
          path: 'date',
          operator: FilterOperator.EQ,
          value1: sKey
        }));

      oBinding.filter(aFilters);
    }

  });
});