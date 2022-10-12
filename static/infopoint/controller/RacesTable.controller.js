sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "sap/ui/core/routing/History",
  "../model/Formatter"
], function (Controller, JSONModel, History, Formatter) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.RacesTable", {

    formatter: Formatter,

    onInit: function () {
      const oComponent = this.getOwnerComponent();

      this.getView().addStyleClass(oComponent.getContentDensityClass());

      oComponent.getRouter().getRoute("races").attachPatternMatched(this._loadRacesModel, this);
    },

    onItemPress: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("races");
        const oRace = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());
        this.getOwnerComponent().setModel(new JSONModel(oRace), "race");

        this.getOwnerComponent().getRouter().navTo("raceRegistrations", {}, false /* history */);
      }
    },

    onNavBack: function () {
      const sPreviousHash = History.getInstance().getPreviousHash();
      if (sPreviousHash) {
        window.history.go(-1);
      } else {
        this.getOwnerComponent().getRouter().navTo("startpage", {}, false /* history */);
      }
    },

    _loadRacesModel: function () {
      const oRegatta = this.getOwnerComponent().getModel("regatta").getData();
      const oRacesModel = new JSONModel();
      oRacesModel.loadData("/api/regattas/" + oRegatta.id + "/races");
      this.getOwnerComponent().setModel(oRacesModel, "races");
    }

  });
});