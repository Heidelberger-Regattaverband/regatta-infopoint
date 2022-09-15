sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/f/library"
], function (Controller, fioriLibrary) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.HeatRegistrationsTable", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());
    },

    handleClose: function () {
      var oFCL = this.getView().getParent().getParent();
      oFCL.setLayout(fioriLibrary.LayoutType.OneColumn);
    }

  });
});