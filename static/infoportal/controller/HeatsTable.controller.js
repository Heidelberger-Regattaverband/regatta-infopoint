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

      this.getEventBus().subscribe("heat", "first", this._onFirstHeatEvent, this);
      this.getEventBus().subscribe("heat", "previous", this._onPreviousHeatEvent, this);
      this.getEventBus().subscribe("heat", "next", this._onNextHeatEvent, this);
      this.getEventBus().subscribe("heat", "last", this._onLastHeatEvent, this);

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
    },

    onNavBack: function () {
      this.oHeatsModel = undefined;
      // reduce table growing threshold to improve performance next time table is shown
      this.heatsTable.setGrowingThreshold(30);
      this.navBack("startpage");
    },

    _onFirstHeatEvent: function (channelId, eventId, parametersMap) {
      this._setCurrentHeat(0);
    },

    _onPreviousHeatEvent: function (channelId, eventId, parametersMap) {
      const iIndex = this.heatsTable.indexOfItem(this.heatsTable.getSelectedItem());
      const iPreviousIndex = iIndex > 1 ? iIndex - 1 : 0;

      if (iIndex != iPreviousIndex) {
        this._setCurrentHeat(iPreviousIndex);
      }
    },

    _onNextHeatEvent: function (channelId, eventId, parametersMap) {
      const aHeats = this.getViewModel("heats").getData();

      const iIndex = this.heatsTable.indexOfItem(this.heatsTable.getSelectedItem());
      const iNextIndex = iIndex < aHeats.length - 1 ? iIndex + 1 : iIndex;

      if (iIndex != iNextIndex) {
        this._growTable(iNextIndex);
        this._setCurrentHeat(iNextIndex);
      }
    },

    _onLastHeatEvent: function (channelId, eventId, parametersMap) {
      const aHeats = this.getViewModel("heats").getData();
      const iIndex = aHeats.length - 1;
      this._growTable(iIndex);
      this._setCurrentHeat(iIndex);
    },

    _loadHeatsModel: async function () {
      if (!this.oHeatsModel) {
        this.oHeatsModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/heats", this.heatsTable);
        this.setViewModel(this.oHeatsModel, "heats");
      }
    },

    _loadRegistrationsModel: async function (sHeatId) {
      const oModel = await this.getJSONModel("/api/heats/" + sHeatId + "/registrations", null);
      this.getOwnerComponent().setModel(oModel, "heatRegistrations");
    },

    _setCurrentHeat: function (iIndex) {
      this.heatsTable.setSelectedItem(this.heatsTable.getItems()[iIndex]);
      const oHeat = this.getViewModel("heats").getData()[iIndex];
      this.getOwnerComponent().getModel("heat").setData(oHeat);
      this._loadRegistrationsModel(oHeat.id);
    },

    _growTable: function (iIndex) {
      const iActual = this.heatsTable.getGrowingInfo().actual;
      if (iIndex >= iActual) {
        this.heatsTable.setGrowingThreshold(iIndex + 10);
        this.heatsTable.getBinding("items").filter([]);
      }
    }

  });
});