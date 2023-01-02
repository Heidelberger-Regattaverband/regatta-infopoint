sap.ui.define([
  "de/regatta_hd/infopoint/controller/BaseTable.controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/model/Filter",
  "../model/Formatter"
], function (BaseTableController, JSONModel, Filter, Formatter) {
  "use strict";

  return BaseTableController.extend("de.regatta_hd.infopoint.controller.RacesTable", {

    formatter: Formatter,

    onInit: function () {
      BaseTableController.prototype.onInit(this.getView().byId("racesTable"));

      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("races").attachMatched(this._loadRacesModel, this);

      this.getEventBus().subscribe("race", "first", this.onFirstItemEvent, this);
      this.getEventBus().subscribe("race", "previous", this.onPreviousItemEvent, this);
      this.getEventBus().subscribe("race", "next", this.onNextItemEvent, this);
      this.getEventBus().subscribe("race", "last", this.onLastItemEvent, this);
    },

    onItemPress: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("races");

        const oRace = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());
        this.getOwnerComponent().setModel(new JSONModel(oRace), "race");

        this._loadRegistrationsModel(oRace.id);
        this.displayTarget("raceRegistrations");
      }
    },

    onNavBack: function () {
      this._oRacesModel = undefined;
      // reduce table growing threshold to improve performance next time table is shown
      this.oTable.setGrowingThreshold(30);
      this.navBack("startpage");
    },

    handleFilterButtonPressed: function (oEvent) {
      this.getViewSettingsDialog("de.regatta_hd.infopoint.view.RacesFilterDialog")
        .then(function (oViewSettingsDialog) {
          oViewSettingsDialog.open();
        });
    },

    _loadRacesModel: async function () {
      if (!this._oRacesModel) {
        this._oRacesModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/races", this.oTable);
        this.setViewModel(this._oRacesModel, "races");
      }
    },

    _loadRegistrationsModel: async function (sRaceId) {
      const oModel = await this.getJSONModel("/api/races/" + sRaceId + "/registrations", undefined);
      this.getOwnerComponent().setModel(oModel, "raceRegistrations");
    },

    setCurrentItem: function (iIndex) {
      this.oTable.setSelectedItem(this.oTable.getItems()[iIndex]);
      const oRace = this.oTable.getSelectedItem().getBindingContext("races").getObject();
      this.getOwnerComponent().getModel("race").setData(oRace);
      this._loadRegistrationsModel(oRace.id);
    },

    handleFilterDialogConfirm: function (oEvent) {
      const mParams = oEvent.getParameters();
      const oBinding = this.oTable.getBinding("items");
      const aFilters = [];

      mParams.filterItems.forEach(function (oItem) {
        const aSplit = oItem.getKey().split("___"),
          sPath = aSplit[0],
          sOperator = aSplit[1],
          sValue1 = aSplit[2] === 'true' || (aSplit[2] === 'false' ? false : aSplit[2]),
          //					sValue2 = aSplit[3],
          oFilter = new Filter(sPath, sOperator, sValue1);
        aFilters.push(oFilter);
      });

      // apply filter settings
      oBinding.filter(aFilters);
      this.setFilters(aFilters);

      // update filter bar
      this.byId("vsdFilterBar").setVisible(aFilters.length > 0);
      this.byId("vsdFilterLabel").setText(mParams.filterString);
    }

  });
});