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
      this.getRouter().navTo("heats", {}, false /* history */);
    },

    onNavToScoring: function () {
      this.getRouter().navTo("scoring", {}, false /* history */);
    },

    onNavToRaces: function () {
      this.getRouter().navTo("races", {}, false /* history */);
    },

    getRouter: function () {
      return this.getOwnerComponent().getRouter();
    }

  });
});