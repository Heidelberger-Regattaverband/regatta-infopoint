sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  'sap/ui/core/Fragment',
  "sap/ui/model/Filter",
  "../model/Formatter"
], function (BaseController, JSONModel, Fragment, Filter, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.RacesTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("races").attachMatched(this._loadRacesModel, this);

      this.getEventBus().subscribe("race", "first", this._onFirstRaceEvent, this);
      this.getEventBus().subscribe("race", "previous", this._onPreviousRaceEvent, this);
      this.getEventBus().subscribe("race", "next", this._onNextRaceEvent, this);
      this.getEventBus().subscribe("race", "last", this._onLastRaceEvent, this);

      this.racesTable = this.getView().byId("racesTable");

      // Keeps reference to any of the created sap.m.ViewSettingsDialog-s in this sample
			this._mViewSettingsDialogs = {};
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
      this.racesTable.setGrowingThreshold(30);
      this.navBack("startpage");
    },

		handleFilterButtonPressed: function(oEvent) {
      this._getViewSettingsDialog("de.regatta_hd.infopoint.view.RacesFilterDialog")
        .then(function (oViewSettingsDialog) {
          oViewSettingsDialog.open();
        });
		},

    _getViewSettingsDialog: function (sDialogFragmentName) {
			let pDialog = this._mViewSettingsDialogs[sDialogFragmentName];

			if (!pDialog) {
        const sStyleClass = this.getOwnerComponent().getContentDensityClass();
				pDialog = Fragment.load({
					id: this.getView().getId(),
					name: sDialogFragmentName,
					controller: this
				}).then(function (oDialog) {
					oDialog.addStyleClass(sStyleClass);
					return oDialog;
				});
				this._mViewSettingsDialogs[sDialogFragmentName] = pDialog;
			}
			return pDialog;
		},

    _onFirstRaceEvent: function (channelId, eventId, parametersMap) {
      this._setCurrentRace(0);
    },

    _onPreviousRaceEvent: function (channelId, eventId, parametersMap) {
      const iIndex = this.racesTable.indexOfItem(this.racesTable.getSelectedItem());
      const iPreviousIndex = iIndex > 1 ? iIndex - 1 : 0;

      if (iIndex != iPreviousIndex) {
        this._setCurrentRace(iPreviousIndex);
      }
    },

    _onNextRaceEvent: function (channelId, eventId, parametersMap) {
      const aRaces = this.getViewModel("races").getData();

      const iIndex = this.racesTable.indexOfItem(this.racesTable.getSelectedItem());
      const iNextIndex = iIndex < aRaces.length - 1 ? iIndex + 1 : iIndex;

      if (iIndex != iNextIndex) {
        this._growTable(iNextIndex);
        this._setCurrentRace(iNextIndex);
      }
    },

    _onLastRaceEvent: function (channelId, eventId, parametersMap) {
      const aRaces = this.getViewModel("races").getData();
      const iIndex = aRaces.length - 1;
      this._growTable(iIndex);
      this._setCurrentRace(iIndex);
    },

    _loadRacesModel: async function () {
      if (!this._oRacesModel) {
        this._oRacesModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/races", this.racesTable);
        this.setViewModel(this._oRacesModel, "races");
      }
    },

    _loadRegistrationsModel: async function (sRaceId) {
      const oModel = await this.getJSONModel("/api/races/" + sRaceId + "/registrations", undefined);
      this.getOwnerComponent().setModel(oModel, "raceRegistrations");
    },

    _setCurrentRace: function (iIndex) {
      this.racesTable.setSelectedItem(this.racesTable.getItems()[iIndex]);
      const oRace = this.getViewModel("races").getData()[iIndex];
      this.getOwnerComponent().getModel("race").setData(oRace);
      this._loadRegistrationsModel(oRace.id);
    },

    _growTable: function (iIndex) {
      const iActual = this.racesTable.getGrowingInfo().actual;
      if (iIndex >= iActual) {
        this.racesTable.setGrowingThreshold(iIndex + 10);
        this.racesTable.getBinding("items").filter([]);
      }
    },

		handleFilterDialogConfirm: function(oEvent) {
			const mParams = oEvent.getParameters();
			const oBinding = this.racesTable.getBinding("items");
			const aFilters = [];

			mParams.filterItems.forEach(function(oItem) {
				const aSplit = oItem.getKey().split("___"),
					sPath = aSplit[0],
					sOperator = aSplit[1],
					sValue1 = true, //aSplit[2],
//					sValue2 = aSplit[3],
					oFilter = new Filter(sPath, sOperator, sValue1);
				aFilters.push(oFilter);
			});

			// apply filter settings
			oBinding.filter(aFilters);

			// update filter bar
			// this.byId("vsdFilterBar").setVisible(aFilters.length > 0);
			// this.byId("vsdFilterLabel").setText(mParams.filterString);
		}

  });
});