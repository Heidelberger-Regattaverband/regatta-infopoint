sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "../model/Formatter"
], function (Controller, Formatter) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.Heats", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());
    },

    onNavBack: function () {
      let oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("startpage", {}, true);
    }
  });
});