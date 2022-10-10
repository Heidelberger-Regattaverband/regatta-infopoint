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
      const oComponent = this.getOwnerComponent();

      this.getView().addStyleClass(oComponent.getContentDensityClass());

      oComponent.getRouter().attachRouteMatched(function (oEvent) {
        if (oEvent.getParameter("name") === "heats") {
          const oRegatta = oComponent.getModel("regatta").getData();
          this._loadHeatsModel(oRegatta);
        }
      }, this);

      // let oRegatta = this.getOwnerComponent().getModel("regatta").getData();

      // let oIconTabBar = this.byId("heatsIconTabBar");
      // let sKey = oIconTabBar.getSelectedKey();
      // this._setFilter(sKey);
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

        this.getOwnerComponent().getRouter().navTo("heatRegistrations");

        // const oFCL = this.getView().getParent().getParent();
        // oFCL.setLayout(fioriLibrary.LayoutType.TwoColumnsMidExpanded);
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
    },

    onNavBack: function () {
      const oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("startpage", {}, true);
    },

    _loadHeatsModel: function (oRegatta) {
      const oHeatsModel = new JSONModel();
      oHeatsModel.loadData("/api/regattas/" + oRegatta.id + "/heats");

      this.getOwnerComponent().setModel(oHeatsModel, "heats");
    }
  });
});