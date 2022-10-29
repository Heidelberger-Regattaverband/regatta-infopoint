sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  'sap/ui/model/FilterOperator',
  "../model/Formatter"
], function (BaseController, JSONModel, Filter, FilterOperator, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.HeatsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getEventBus().subscribe("heat", "previous", this.previousHeat, this);
      this.getEventBus().subscribe("heat", "next", this.nextHeat, this);

      this.getRouter().getRoute("heats").attachMatched(function () {
        this.byId("heatsIconTabBar").setSelectedKey("all");
        this._loadHeatsModel();
      }, this);

      this.heatsTable = this.getView().byId("heatsTable");
    },

    onSelectionChange: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("heats");
        const oHeat = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());
        this.getOwnerComponent().setModel(new JSONModel(oHeat), "heat");

        this.getRouter().navTo("heatRegistrations");
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
      this.oHeatsModel = undefined;
      this.navBack("startpage");
    },

    previousHeat: function (channelId, eventId, parametersMap) {
      const iIndex = this.heatsTable.indexOfItem(this.heatsTable.getSelectedItem());
      const iPreviousIndex = iIndex > 1 ? iIndex - 1 : 0;

      if (iIndex != iPreviousIndex) {
        this.heatsTable.setSelectedItem(this.heatsTable.getItems()[iPreviousIndex]);
        const oHeat = this.getViewModel("heats").getData()[iPreviousIndex];
        this.getOwnerComponent().getModel("heat").setData(oHeat);
      }
    },

    nextHeat: function (channelId, eventId, parametersMap) {
      const aHeats = this.getViewModel("heats").getData();

      this._growTable(aHeats);

      const iIndex = this.heatsTable.indexOfItem(this.heatsTable.getSelectedItem());
      const iNextIndex = iIndex < aHeats.length - 1 ? iIndex + 1 : iIndex;

      if (iIndex != iNextIndex) {
        this.heatsTable.setSelectedItem(this.heatsTable.getItems()[iNextIndex]);
        this.getOwnerComponent().getModel("heat").setData(aHeats[iNextIndex]);
      }
    },

    _loadHeatsModel: function () {
      if (!this.oHeatsModel) {
        const oRegatta = this.getOwnerComponent().getModel("regatta").getData();
        this.oHeatsModel = new JSONModel();
        this.oHeatsModel.loadData("/api/regattas/" + oRegatta.id + "/heats");
        this.setViewModel(this.oHeatsModel, "heats");
      }
    },

    _growTable: function (aHeats) {
      const iActual = this.heatsTable.getGrowingInfo().actual;
      if (aHeats.length > iActual) {
        this.heatsTable.setGrowingThreshold(iActual + 20);
        this.heatsTable.getBinding("items").filter([]);
      }
    }
  });
});