sap.ui.define([
  "de/regatta_hd/infopoint/controller/BaseTable.controller",
  "sap/ui/model/Filter",
  "sap/ui/model/FilterOperator",
  "sap/m/MessageToast",
  "sap/m/ViewSettingsItem",
  "sap/m/ViewSettingsFilterItem",
  "../model/Formatter"],
  function (BaseTableController, Filter, FilterOperator, MessageToast, ViewSettingsItem, ViewSettingsFilterItem, Formatter) {
    "use strict";

    return BaseTableController.extend("de.regatta_hd.infopoint.controller.HeatsTable", {

      formatter: Formatter,

      onInit: async function () {
        this.init(this.getView().byId("heatsTable"), "heat" /* eventBus channel */);

        this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

        this._oHeatsModel = await this.getJSONModel(`/api/regattas/${this.getRegattaId()}/heats`, this.table);
        this.setViewModel(this._oHeatsModel, "heats");

        this.getRouter().getRoute("heats").attachMatched(async (_) => await this._loadHeatsModel(), this);

        const mFilters = this.getComponentModel("filters").getData();

        // initialize filter values
        const oViewSettingsDialog = await this.getViewSettingsDialog("de.regatta_hd.infopoint.view.HeatsFilterDialog");

        if (mFilters.dates) {
          const oDatesFilter = new ViewSettingsFilterItem({ multiSelect: true, key: "day", text: "{i18n>common.day}" });
          mFilters.dates.forEach((sDate) => {
            oDatesFilter.addItem(new ViewSettingsItem({ text: Formatter.weekDayDateLabel(sDate), key: "dateTime___Contains___" + sDate }));
          });
          oViewSettingsDialog.insertFilterItem(oDatesFilter, 0);
        }

        if (mFilters.rounds) {
          const oRoundFilter = new ViewSettingsFilterItem({ multiSelect: true, key: "round", text: "{i18n>common.round}" });
          mFilters.rounds.forEach((mRound) => {
            oRoundFilter.addItem(new ViewSettingsItem({ text: Formatter.roundLabel(mRound.code), key: "roundCode___EQ___" + mRound.code }))
          });
          oViewSettingsDialog.insertFilterItem(oRoundFilter, 1);
        }

        if (mFilters.boatClasses) {
          const oBoatClassFilter = new ViewSettingsFilterItem({ multiSelect: true, key: "boatClass", text: "{i18n>common.boatClass}" });
          mFilters.boatClasses.forEach((boatClass) => {
            oBoatClassFilter.addItem(new ViewSettingsItem({ text: boatClass.caption + " (" + boatClass.abbreviation + ")", key: "race/boatClass/id___EQ___" + boatClass.id }));
          });
          oViewSettingsDialog.insertFilterItem(oBoatClassFilter, 2);
        }

        if (mFilters.ageClasses) {
          const oAgeClassFilter = new ViewSettingsFilterItem({ multiSelect: true, key: "ageClass", text: "{i18n>common.ageClass}" });
          mFilters.ageClasses.forEach((ageClass) => {
            oAgeClassFilter.addItem(new ViewSettingsItem({ text: ageClass.caption + " " + ageClass.suffix + "", key: "race/ageClass/id___EQ___" + ageClass.id }));
          });
          oViewSettingsDialog.insertFilterItem(oAgeClassFilter, 3);
        }

        if (mFilters.distances && mFilters.distances.length > 1) {
          const oDistancesFilter = new ViewSettingsFilterItem({ multiSelect: true, key: "distance", text: "{i18n>common.distance}" });
          mFilters.distances.forEach((distance) => {
            oDistancesFilter.addItem(new ViewSettingsItem({ text: distance + "m", key: "race/distance___EQ___" + distance }));
          });
          oViewSettingsDialog.insertFilterItem(oDistancesFilter, 5);
        }
      },

      onSelectionChange: function (oEvent) {
        const oSelectedItem = oEvent.getParameter("listItem");
        if (oSelectedItem) {
          const oBindingCtx = oSelectedItem.getBindingContext("heats");
          const oHeat = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());

          const iIndex = this.table.indexOfItem(oSelectedItem);
          const iCount = this.table.getItems().length;
          // store navigation meta information in selected item
          oHeat._nav = { isFirst: iIndex == 0, isLast: iIndex == iCount - 1 };

          this.onItemChanged(oHeat);
          this.displayTarget("heatRegistrations");
        }
      },

      onNavBack: async function () {
        await this.navBack("startpage");

        // reduce table growing threshold to improve performance next time table is shown
        this.table.setGrowingThreshold(30);
      },

      onRefreshButtonPress: async function (oEvent) {
        const oSource = oEvent.getSource();
        oSource.setEnabled(false);
        await this._loadHeatsModel();
        MessageToast.show(this.i18n("msg.dataUpdated", undefined));
        oSource.setEnabled(true);
      },

      _loadHeatsModel: async function () {
        await this.updateJSONModel(this._oHeatsModel, `/api/regattas/${this.getRegattaId()}/heats`, this.table);
      },

      onItemChanged: function (oItem) {
        this.getComponentModel("heat").setData(oItem);
        this.getEventBus().publish("heat", "itemChanged", {});
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