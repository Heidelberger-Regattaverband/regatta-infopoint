sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/core/Fragment"
], function (BaseController, Fragment) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.BaseTable", {

    onInit: function () {
      // Keeps reference to any of the created sap.m.ViewSettingsDialog-s in this sample
      this._mViewSettingsDialogs = {};
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
    }

  });

});