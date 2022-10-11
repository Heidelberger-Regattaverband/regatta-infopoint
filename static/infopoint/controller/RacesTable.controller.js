sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "../model/Formatter"
], function (Controller, JSONModel, Formatter) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.RacesTable", {

    formatter: Formatter,

    onInit: function () {
      const oComponent = this.getOwnerComponent();

      this.getView().addStyleClass(oComponent.getContentDensityClass());

      this.getOwnerComponent().getRouter().attachRouteMatched(function (oEvent) {
        if (oEvent.getParameter("name") === "races") {
          const oRegatta = oComponent.getModel("regatta").getData();
          this._loadRacesModel(oRegatta);
        }
      }, this);
    },

    onNavBack: function () {
      const oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("startpage", {}, true);
    },

    _loadRacesModel: function (oRegatta) {
      const oRacesModel = new JSONModel();
      oRacesModel.loadData("/api/regattas/" + oRegatta.id + "/races");
      this.getOwnerComponent().setModel(oRacesModel, "races");
    }
  });
});