sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "../model/Formatter"
], function (BaseController, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.HeatRegistrationsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());
    },

    onNavBack: function () {
      this.displayTarget("heats");
    },

    handlePrevious: function () {
      this.getEventBus().publish("heat", "previous", {});
    },

    handleNext: function () {
      this.getEventBus().publish("heat", "next", {});
    }

  });
});