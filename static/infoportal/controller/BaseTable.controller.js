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

      const iIndex = this.oTable.getItems().length - 1;
      this._setCurrentItem(iIndex);
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
      let aItems = this.oTable.getItems();
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
      const mParams = oEvent.getParameters(),
        oBinding = this.oTable.getBinding("items"),
        aFilters = [];

      mParams.filterItems.forEach(function (oItem) {
        const aSplit = oItem.getKey().split("___"),
          sPath = aSplit[0],
          sOperator = aSplit[1],
          sValue1 = aSplit[2] === 'true' || (aSplit[2] === 'false' ? false : aSplit[2]),
          // sValue2 = aSplit[3],
          oFilter = new Filter(sPath, sOperator, sValue1);
        aFilters.push(oFilter);
      });

      // apply filter settings
      oBinding.filter(aFilters);
      this.setFilters(aFilters);

      // update filter bar
      const oInfoToolbar = this.oTable.getInfoToolbar();
      if (oInfoToolbar && oInfoToolbar.getContent()[0]) {
        oInfoToolbar.setVisible(aFilters.length > 0);
        oInfoToolbar.getContent()[0].setText(mParams.filterString);
      }
    },

    setFilters: function (aFilters = []) {
      this._aFilters = aFilters;
    },

    _setCurrentItem: function (iIndex) {
      this.oTable.setSelectedItem(this.oTable.getItems()[iIndex]);
      const sBindingModel = this.oTable.getBindingInfo("items").model;
      const oItem = this.oTable.getSelectedItem().getBindingContext(sBindingModel).getObject();
      this.onItemChanged(oItem);
    },

    onItemChanged: function (oItem) {
    },

    _growTable: function (iIndex) {
      const iActual = this.oTable.getGrowingInfo().actual;
      if (iIndex >= iActual) {
        this.oTable.setGrowingThreshold(iIndex + 10);
        this.oTable.getBinding("items").filter(this._aFilters);
      }
    }
  });

});