sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/Filter",
  "sap/ui/core/Fragment"
], function (BaseController, Filter, Fragment) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.BaseTable", {

    init: function (oTable, sChannelId) {
      // Keeps reference to any of the created sap.m.ViewSettingsDialog-s in this sample
      this._mViewSettingsDialogs = {};

      this.oTable = oTable;
      this._aFilters = [];
      this._aSearchFilters = [];

      // return the path of the model that is bound to the items, e.g. races or heats
      this._sBindingModel = this.oTable.getBindingInfo("items").model;

      this.getEventBus().subscribe(sChannelId, "first", this._onFirstItemEvent, this);
      this.getEventBus().subscribe(sChannelId, "previous", this._onPreviousItemEvent, this);
      this.getEventBus().subscribe(sChannelId, "next", this._onNextItemEvent, this);
      this.getEventBus().subscribe(sChannelId, "last", this._onLastItemEvent, this);
    },

    _onFirstItemEvent: function (channelId, eventId, parametersMap) {
      const iIndex = this.oTable.indexOfItem(this.oTable.getSelectedItem());
      if (iIndex != 0) {
        this._setCurrentItem(0);
      }
    },

    _onLastItemEvent: function (channelId, eventId, parametersMap) {
      this._growTable(400);
      const iIndex = this.oTable.indexOfItem(this.oTable.getSelectedItem());
      const iLastIndex = this.oTable.getItems().length - 1;
      if (iIndex != iLastIndex) {
        this._setCurrentItem(iLastIndex);
      }
    },

    _onPreviousItemEvent: function (channelId, eventId, parametersMap) {
      const iIndex = this.oTable.indexOfItem(this.oTable.getSelectedItem());
      const iPreviousIndex = iIndex > 1 ? iIndex - 1 : 0;

      if (iIndex != iPreviousIndex) {
        this._setCurrentItem(iPreviousIndex);
      }
    },

    _onNextItemEvent: function (channelId, eventId, parametersMap) {
      const iIndex = this.oTable.indexOfItem(this.oTable.getSelectedItem());
      const aItems = this.oTable.getItems();
      const iNextIndex = iIndex < aItems.length - 1 ? iIndex + 1 : iIndex;
      if (iIndex != iNextIndex) {
        this._growTable(iNextIndex);
        this._setCurrentItem(iNextIndex);
      }
    },

    getViewSettingsDialog: function (sDialogFragmentName) {
      let pDialog = this._mViewSettingsDialogs[sDialogFragmentName];

      if (!pDialog) {
        const sStyleClass = this.getOwnerComponent().getContentDensityClass();
        const oView = this.getView();
        pDialog = Fragment.load({
          id: this.getView().getId(),
          name: sDialogFragmentName,
          controller: this
        }).then(function (oDialog) {
          oDialog.addStyleClass(sStyleClass);
          oView.addDependent(oDialog);
          return oDialog;
        });
        this._mViewSettingsDialogs[sDialogFragmentName] = pDialog;
      }
      return pDialog;
    },

    onHandleFilterDialogConfirm: function (oEvent) {
      const mParams = oEvent.getParameters();
      this._aFilters = [];
      const that = this;

      mParams.filterItems.forEach(function (oItem) {
        const aCustomData = oItem.getCustomData();
        if (aCustomData) {
          aCustomData.forEach(function (oData) {
            if (oData.getKey() == "filter") {
              const oFilter = that._createFilter(oData.getValue());
              that._aFilters.push(oFilter);
            }
          });
        }
        const oFilter = that._createFilter(oItem.getKey());
        that._aFilters.push(oFilter);
      });

      // apply filters
      this.applyFilters();

      // update filter bar
      const oInfoToolbar = this.oTable.getInfoToolbar();
      if (oInfoToolbar && oInfoToolbar.getContent()[0]) {
        oInfoToolbar.setVisible(this._aFilters.length > 0);
        oInfoToolbar.getContent()[0].setText(mParams.filterString);
      }
    },

    _createFilter: function (sValue) {
      const aSplit = sValue.split("___"),
        sPath = aSplit[0],
        sOperator = aSplit[1],
        sValue1 = aSplit[2] === 'true' || (aSplit[2] === 'false' ? false : aSplit[2]),
        // sValue2 = aSplit[3],
        oFilter = new Filter(sPath, sOperator, sValue1);
      return oFilter;
    },

    setSearchFilters: function (aSearchFilters = []) {
      this._aSearchFilters = aSearchFilters
    },

    applyFilters: function () {
      // combine search and filters from dialog
      const aAllFilters = this._aFilters.concat(this._aSearchFilters)
      const oBinding = this.oTable.getBinding("items");
      oBinding.filter(aAllFilters);
    },

    _setCurrentItem: function (iIndex) {
      const aItems = this.oTable.getItems();
      this.oTable.setSelectedItem(aItems[iIndex]);

      // gets the selected item in a generic way
      const oItem = this.oTable.getSelectedItem().getBindingContext(this._sBindingModel).getObject();

      // store navigation meta information in selected item
      oItem._nav = { isFirst: iIndex == 0, isLast: iIndex == aItems.length - 1 };

      this.onItemChanged(oItem);
    },

    onItemChanged: function (oItem) {
    },

    _growTable: function (iIndex) {
      const iActual = this.oTable.getGrowingInfo().actual;
      if (iIndex >= iActual) {
        this.oTable.setGrowingThreshold(iIndex + 10);
        const aAllFilters = this._aFilters.concat(this._aSearchFilters)
        this.oTable.getBinding("items").filter(aAllFilters);
      }
    }
  });

});