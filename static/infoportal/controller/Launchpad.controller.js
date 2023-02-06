sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "../model/Formatter"
], function (BaseController, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Launchpad", {

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

    onNavToStatistics: function () {
      this.getRouter().navTo("statistics", {}, false /* history */);
    },

    onNavToKiosk: function () {
      this.getRouter().navTo("kiosk", {}, false /* history */);
    }

  });
});