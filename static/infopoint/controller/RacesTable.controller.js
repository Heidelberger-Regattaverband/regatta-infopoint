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

      oComponent.getRouter().getRoute("races")
        .attachPatternMatched(this._loadRacesModel, this);
    },

    onNavBack: function () {
      const oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("startpage", {}, true);
    },

    _loadRacesModel: function () {
      const oRegatta = this.getOwnerComponent().getModel("regatta").getData();
      const oRacesModel = new JSONModel();
      oRacesModel.loadData("/api/regattas/" + oRegatta.id + "/races");
      this.getOwnerComponent().setModel(oRacesModel, "races");
    }

  });
});