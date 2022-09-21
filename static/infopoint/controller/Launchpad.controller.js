sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "../model/Formatter"
], function (Controller, Formatter) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.Launchpad", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());
    },

    onNavToHeats: function () {
      this.getRouter().navTo("heats");
    },

    getRouter: function () {
      return this.getOwnerComponent().getRouter();
    }

  });
});