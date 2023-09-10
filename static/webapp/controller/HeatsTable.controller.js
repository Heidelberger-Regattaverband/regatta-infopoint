sap.ui.define([
  "de/regatta_hd/infopoint/controller/BaseTable.controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  "sap/ui/model/FilterOperator",
  "sap/m/MessageToast",
  "sap/m/ViewSettingsItem",
  "../model/Formatter"], function (BaseTableController, JSONModel, Filter, FilterOperator, MessageToast, ViewSettingsItem, Formatter) {
    "use strict";

    return BaseTableController.extend("de.regatta_hd.infopoint.controller.HeatsTable", {

      formatter: Formatter,

      onInit: async function () {
        this.init(this.getView().byId("heatsTable"), "heat");

        this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

        this._oHeatsModel = await this.getJSONModel(`/api/regattas/${this.getRegattaId()}/heats`, this.oTable);
        this.setViewModel(this._oHeatsModel, "heats");

        this._oHeatModel = new JSONModel();
        this.getOwnerComponent().setModel(this._oHeatModel, "heatRegistrations");

        this.getRouter().getRoute("heats").attachMatched(async (_) => await this._loadHeatsModel(), this);

        // this.getEventBus().subscribe("heat", "refresh", async (_) => await this._loadHeatsModel(), this);

        const oFilters = this.getOwnerComponent().getModel("filters").getData();

        // initialize filter values
        const oViewSettingsDialog = await this.getViewSettingsDialog("de.regatta_hd.infopoint.view.HeatsFilterDialog");
        oViewSettingsDialog.getFilterItems().forEach(oFilterItem => {
          switch (oFilterItem.getKey()) {
            case 'day':
              oFilters.dates.forEach((sDate) => {
                oFilterItem.addItem(new ViewSettingsItem({ text: Formatter.weekDayDateLabel(sDate), key: "dateTime___Contains___" + sDate }));
              });
              break;
            case 'distance':
              oFilters.distances.forEach((distance) => {
                oFilterItem.addItem(new ViewSettingsItem({ text: distance + "m", key: "race/distance___EQ___" + distance }));
              });
              break;
          }
        });
      },

      onSelectionChange: function (oEvent) {
        const oSelectedItem = oEvent.getParameter("listItem");
        if (oSelectedItem) {
          const oBindingCtx = oSelectedItem.getBindingContext("heats");
          const oHeat = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());

          const iIndex = this.oTable.indexOfItem(oSelectedItem);
          const iCount = this.oTable.getItems().length;
          // store navigation meta information in selected item
          oHeat._nav = { isFirst: iIndex == 0, isLast: iIndex == iCount - 1 };

          this.getOwnerComponent().setModel(new JSONModel(oHeat), "heat");

          this._loadHeatModel(oHeat.id);
          this.displayTarget("heatRegistrations");
        }
      },

      onNavBack: async function () {
        await this.navBack("startpage");

        // reduce table growing threshold to improve performance next time table is shown
        this.oTable.setGrowingThreshold(30);
      },

      onRefreshButtonPress: async function (oEvent) {
        const oSource = oEvent.getSource();
        oSource.setEnabled(false);
        await this._loadHeatsModel();
        MessageToast.show(this.i18n("msg.dataUpdated", undefined));
        oSource.setEnabled(true);
      },

      _loadHeatsModel: async function () {
        await this.updateJSONModel(this._oHeatsModel, `/api/regattas/${this.getRegattaId()}/heats`, this.oTable);
      },

      _loadHeatModel: async function (sHeatId) {
        await this.updateJSONModel(this._oHeatModel, `/api/heats/${sHeatId}`, undefined);
      },

      onItemChanged: function (oItem) {
        this.getOwnerComponent().getModel("heat").setData(oItem);
        this._loadHeatModel(oItem.id);
      },

      onFilterButtonPress: async function (oEvent) {
        const oViewSettingsDialog = await this.getViewSettingsDialog("de.regatta_hd.infopoint.view.HeatsFilterDialog");
        oViewSettingsDialog.open();
      },

      onClearFilterPress: async function (oEvent) {
        const oViewSettingsDialog = await this.getViewSettingsDialog("de.regatta_hd.infopoint.view.HeatsFilterDialog")
        oViewSettingsDialog.clearFilters();
        this.clearFilters();
        this.applyFilters();
      },

      onFilterSearch: function (oEvent) {
        const aSearchFilters = [];
        const sQuery = oEvent.getParameter("query").trim();
        if (sQuery) {
          aSearchFilters.push(
            new Filter({
              filters: [
                new Filter("race/number", FilterOperator.Contains, sQuery),
                new Filter("race/shortLabel", FilterOperator.Contains, sQuery),
                new Filter("race/longLabel", FilterOperator.Contains, sQuery),
                new Filter("race/comment", FilterOperator.Contains, sQuery)
              ],
              and: false
            }))
        }
        this.setSearchFilters(aSearchFilters);
        this.applyFilters();
      }
    });

  });