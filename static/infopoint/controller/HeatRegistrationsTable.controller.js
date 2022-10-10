sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/f/library",
  "../model/Formatter"
], function (Controller, fioriLibrary, Formatter) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.HeatRegistrationsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());
    },

    onNavBack: function () {
      const oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("heats", {}, true);
    },

    handlePrevious: function () {
    },

    handleNext: function () {
    }

  });
});