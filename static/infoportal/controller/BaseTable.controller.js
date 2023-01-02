sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/core/Fragment"
], function (BaseController, Fragment) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.BaseTable", {

    onInit: function (oTable) {
      // Keeps reference to any of the created sap.m.ViewSettingsDialog-s in this sample
      this._mViewSettingsDialogs = {};

      this.oTable = oTable;
      this._aFilters = [];
    },

    onFirstItemEvent: function (channelId, eventId, parametersMap) {
      this.setCurrentItem(0);
    },

    onLastItemEvent: function (channelId, eventId, parametersMap) {
      this._growTable(400);

      const iIndex = this.oTable.getItems().length - 1;
      this.setCurrentItem(iIndex);
    },

    onPreviousItemEvent: function (channelId, eventId, parametersMap) {
      const iIndex = this.oTable.indexOfItem(this.oTable.getSelectedItem());
      const iPreviousIndex = iIndex > 1 ? iIndex - 1 : 0;

      if (iIndex != iPreviousIndex) {
        this.setCurrentItem(iPreviousIndex);
      }
    },

    onNextItemEvent: function (channelId, eventId, parametersMap) {
      const iIndex = this.oTable.indexOfItem(this.oTable.getSelectedItem());
      let aItems = this.oTable.getItems();
      const iNextIndex = iIndex < aItems.length - 1 ? iIndex + 1 : iIndex;
      if (iIndex != iNextIndex) {
        this._growTable(iNextIndex);
        this.setCurrentItem(iNextIndex);
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

    setFilters: function (aFilters = []) {
      this._aFilters = aFilters;
    },

    setCurrentItem: function (iIndex) {
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