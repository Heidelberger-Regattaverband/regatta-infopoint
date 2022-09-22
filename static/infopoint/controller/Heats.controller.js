sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "../model/Formatter"
], function (Controller, JSONModel, Formatter) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.Heats", {

    formatter: Formatter,

    onInit: function () {
      const oComponent = this.getOwnerComponent();

      this.getView().addStyleClass(oComponent.getContentDensityClass());

      oComponent.getRouter().attachRouteMatched(function (oEvent) {
        if (oEvent.getParameter("name") === "heats") {
          const oRegatta = oComponent.getModel("regatta").getData();
          this._loadHeatsModel(oRegatta);
        }
      }, this);
    },

    onNavBack: function () {
      const oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("startpage", {}, true);
    },

    _loadHeatsModel: function (oRegatta) {
      const oHeatsModel = new JSONModel();
      oHeatsModel.loadData("/api/regattas/" + oRegatta.id + "/heats");

      this.getOwnerComponent().setModel(oHeatsModel, "heats");
    }

  });
});